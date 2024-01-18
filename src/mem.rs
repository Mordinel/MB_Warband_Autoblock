use process_memory::{ProcessHandle, DataMember, Memory};

pub fn resolve_pointer_path<T: Copy>(
    handle: &ProcessHandle,
    base_addr: usize,
    pointer_path: &[usize]
) -> std::io::Result<DataMember<T>> {
    if pointer_path.is_empty() {
        return Err(std::io::ErrorKind::InvalidData.into());
    }
    let mut result_addr = base_addr;

    // iterate over each pointer offset except the last one
    for offset in pointer_path[0..pointer_path.len()-1].iter() {
        let member: DataMember<usize> = DataMember::new_offset(*handle, vec![result_addr + *offset as usize]);
        match unsafe {
            member.read()
        } {
            Ok(value) => {
                result_addr = value;
            },
            Err(why) => {
                eprintln!("Failed to read from member: {why}\r");
                return Err(std::io::ErrorKind::AddrNotAvailable.into());
            },
        }
    }

    // last offset is read as T rather than usize (memory address)
    let member: DataMember<T> = DataMember::new_offset(
        *handle,
        vec![result_addr + *pointer_path.iter().last().unwrap() as usize]
    );
    Ok(member)
}

