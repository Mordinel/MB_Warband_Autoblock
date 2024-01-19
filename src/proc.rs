#[cfg(target_os = "macos")]
use mach2::{
    port::mach_port_t,
    kern_return::KERN_SUCCESS,
    message::mach_msg_type_number_t,
    traps::{task_for_pid, mach_task_self},
    vm::mach_vm_region,
    vm_prot::VM_PROT_EXECUTE,
    vm_types::{vm_address_t, vm_size_t, mach_vm_address_t, mach_vm_size_t},
    vm_region::{vm_region_basic_info, VM_REGION_BASIC_INFO_64, vm_region_info_t},
};

#[cfg(target_os = "windows")]
use winapi::{
    um::{
        processthreadsapi::OpenProcess,
        psapi::{EnumProcessModules, GetModuleFileNameExW},
        winnt::{PROCESS_QUERY_INFORMATION, PROCESS_VM_READ},
    },
    shared::minwindef::MAX_PATH,
};
#[cfg(target_os = "windows")]
use std::ptr;

use process_memory::{ProcessHandle, TryIntoProcessHandle, Pid};
use sysinfo::System;

pub fn get_pid(name: &str) -> std::io::Result<i32> {
    let mut system = System::new_all();
    system.refresh_all();
    let pid = if let Some(proc) = system.processes_by_exact_name(name).next() {
        proc.pid().as_u32() as i32
    } else {
        return Err(std::io::ErrorKind::NotFound.into());
    };
    Ok(pid)
}

pub fn get_handle(pid: i32) -> std::io::Result<ProcessHandle> {
    Ok((pid as Pid).try_into_process_handle()?)
}

#[cfg(target_os = "windows")]
pub fn get_base_address(pid: i32) -> std::io::Result<usize> {
    let process_handle = unsafe {
        OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, pid as u32)
    };

    if process_handle == winapi::um::handleapi::INVALID_HANDLE_VALUE {
        eprintln!("Failed to get process handle for PID: {pid}, You probably lack the right to obtain this (run as Administrator)\r");
        return Err(std::io::ErrorKind::PermissionDenied.into());
    }

    let mut h_mod = [ptr::null_mut(); 1]; 
    let mut cb_needed = 0;

    if unsafe {
        EnumProcessModules(
            process_handle,
            h_mod.as_mut_ptr(),
            std::mem::size_of::<usize>() as u32,
            &mut cb_needed,
        )
    } == 1 {
        let mut module_name = vec![0; MAX_PATH];
        if unsafe {
            GetModuleFileNameExW(
                process_handle,
                h_mod[0],
                module_name.as_mut_ptr(),
                module_name.len() as u32
            )
        } > 0 {
            let base_address = h_mod[0] as usize;
            unsafe { winapi::um::handleapi::CloseHandle(process_handle) };
            Ok(base_address)
        } else {
            unsafe { winapi::um::handleapi::CloseHandle(process_handle) };
            Err(std::io::ErrorKind::NotFound.into())
        }
    } else {
        unsafe { winapi::um::handleapi::CloseHandle(process_handle) };
        Err(std::io::ErrorKind::NotFound.into())
    }
}

#[cfg(target_os = "macos")]
pub fn get_base_address(pid: i32) -> std::io::Result<usize> {
    let mut task: mach_port_t = 0;
    if unsafe {
        task_for_pid(mach_task_self(), pid, &mut task)
    } != KERN_SUCCESS {
        eprintln!("Failed to get task for PID: {pid}, You probably lack the right to obtain this (run as root)\r");
        return Err(std::io::ErrorKind::PermissionDenied.into());
    }

    let mut address: vm_address_t = 1;
    let mut size: vm_size_t = 0;
    let mut info: vm_region_basic_info = unsafe { std::mem::zeroed() };
    let mut info_count = std::mem::size_of_val(&info) as mach_msg_type_number_t;
    let mut obj_name: mach_port_t = 0;

    while unsafe {
        mach_vm_region(
            task, 
            &mut address as *mut _ as *mut mach_vm_address_t,
            &mut size as *mut _ as *mut mach_vm_size_t,
            VM_REGION_BASIC_INFO_64,
            &mut info as *mut _ as vm_region_info_t,
            &mut info_count,
            &mut obj_name
        )
    } == KERN_SUCCESS {
        if info.protection & VM_PROT_EXECUTE != 0 {
            return Ok(address);
        }
        address += size;
    }

    eprintln!("Could not find executable region for PID: {pid}\r");
    Err(std::io::ErrorKind::NotFound.into())
}

