use std::collections::HashMap;

use miette::Result;

use crate::{
    parser::ast::Immediate,
    vm::{Error, VM, register::Register},
};

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
    let path_addr = vm.regs.get(Register::Q0).as_usize()?;
    let flags = vm.regs.get(Register::D1).as_u32()? as i32;
    let mode = vm.regs.get(Register::D2).as_u32()? as libc::c_int;

    if path_addr >= vm.mem.len() {
        return Err(Error::InstructionPointerOutOfBounds(path_addr))?;
    }

    let cstr = unsafe {
        let ptr = vm.mem.storage.as_ptr().add(path_addr);
        std::ffi::CStr::from_ptr(ptr as *const _)
    };

    let fd = unsafe { libc::open(cstr.as_ptr(), flags, mode) };
    if fd < 0 {
        return Err(Error::IoError(std::io::Error::last_os_error()))?;
    }

    vm.regs.set(Register::D0, Immediate::DWord(fd as u32))
}

fn sys_close(vm: &mut VM) -> Result<()> {
    let fd = vm.regs.get(Register::D0).as_u32()? as i32;

    let ret = unsafe { libc::close(fd) };
    if ret < 0 {
        return Err(Error::IoError(std::io::Error::last_os_error()))?;
    }

    vm.regs.set(Register::D0, Immediate::DWord(ret as u32))
}

fn sys_read(vm: &mut VM) -> Result<()> {
    let fd = vm.regs.get(Register::D0).as_u32()? as i32;
    let addr = vm.regs.get(Register::Q1).as_usize()?;
    let count = vm.regs.get(Register::Q2).as_usize()?;

    if addr + count >= vm.mem.len() {
        return Err(Error::InstructionPointerOutOfBounds(addr + count))?;
    }

    let buf = vm.mem.storage[addr..addr + count].as_mut_ptr();

    let n = unsafe { libc::read(fd, buf as *mut _, count) };
    if n < 0 {
        return Err(Error::IoError(std::io::Error::last_os_error()))?;
    }

    vm.regs.set(Register::Q0, Immediate::QWord(n as u64))
}

fn sys_write(vm: &mut VM) -> Result<()> {
    let fd = vm.regs.get(Register::D0).as_u32()? as i32;
    let addr = vm.regs.get(Register::Q1).as_usize()?;
    let count = vm.regs.get(Register::Q2).as_usize()?;

    if addr + count >= vm.mem.len() {
        return Err(Error::InstructionPointerOutOfBounds(addr + count))?;
    }

    let buf = vm.mem.storage[addr..addr + count].as_ptr();

    let n = unsafe { libc::write(fd, buf as *const _, count) };
    if n < 0 {
        return Err(Error::IoError(std::io::Error::last_os_error()))?;
    }

    vm.regs.set(Register::Q0, Immediate::QWord(n as u64))
}
