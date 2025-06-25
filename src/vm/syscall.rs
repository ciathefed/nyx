use std::collections::HashMap;

use miette::Result;

use crate::vm::VM;

pub type SyscallFn = fn(vm: &mut VM) -> Result<()>;
pub type Syscalls = HashMap<usize, SyscallFn>;

pub fn collect_syscalls() -> Syscalls {
    let mut syscalls: Syscalls = HashMap::with_capacity(256);

    syscalls.insert(0x00, sys_open);
    syscalls.insert(0x01, sys_close);
    syscalls.insert(0x02, sys_read);
    syscalls.insert(0x03, sys_write);

    syscalls
}

fn sys_open(vm: &mut VM) -> Result<()> {
    Ok(())
}

fn sys_close(vm: &mut VM) -> Result<()> {
    Ok(())
}

fn sys_read(vm: &mut VM) -> Result<()> {
    Ok(())
}

fn sys_write(vm: &mut VM) -> Result<()> {
    Ok(())
}
