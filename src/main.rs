#![allow(non_camel_case_types)]
use binary_layout::prelude::*;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::os::unix::fs::OpenOptionsExt;

type Elf64_Addr = u64;
type Elf64_Off = u64;
type Elf64_Half = u16;
type Elf64_Word = u32;
// type Elf64_Sword = i32;
type Elf64_Xword = u64;
// type Elf64_Sxword = i64;

define_layout!(elf64_hdr, LittleEndian, {
    ident: [u8; 16],
    _type: Elf64_Half,
    machine: Elf64_Half,
    version: Elf64_Word,
    entry: Elf64_Addr, // virtual address of entry point
    phoff: Elf64_Off, // program header
    shoff: Elf64_Off, // section header
    flags: Elf64_Word, // processor-specific
    ehsize: Elf64_Half,
    phentsize: Elf64_Half,
    phnum: Elf64_Half, // number of program header entries
    shentsize: Elf64_Half, // size of section header entry
    shnum: Elf64_Half, // number of section header entries
    shstrndx: Elf64_Half, // section name string table index
});

fn set_elf64_hdr(storage: &mut [u8]) {
    let mut view = elf64_hdr::View::new(storage);
    view.ident_mut().copy_from_slice(&create_ident());
    view._type_mut().write(2); // ET_EXEC
    view.machine_mut().write(62); // EM_X86_64
    view.version_mut().write(1); // EV_CURRENT
    view.entry_mut().write(VADDR + program_offset());
    view.phoff_mut().write(elf64_hdr::SIZE.unwrap() as u64);
    view.flags_mut().write(0); // no processor-specific flags
    view.ehsize_mut().write(elf64_hdr::SIZE.unwrap() as u16);
    view.phentsize_mut().write(elf64_phdr::SIZE.unwrap() as u16);
    view.phnum_mut().write(1);
}

define_layout!(elf64_phdr, LittleEndian, {
    _type: Elf64_Word,
    flags: Elf64_Word,
    offset: Elf64_Off,
    vaddr: Elf64_Addr,
    paddr: Elf64_Addr,
    filesz: Elf64_Xword,
    memsz: Elf64_Xword,
    align: Elf64_Xword,
});

fn create_ident() -> [u8; 16] {
    let mut ident = [0; 16];
    ident[0] = 0x7f; // magic
    ident[1] = 'E' as u8;
    ident[2] = 'L' as u8;
    ident[3] = 'F' as u8;
    ident[4] = 2; // class: ELFCLASS64
    ident[5] = 1; // data encoding: ELFDATA2LSB
    ident[6] = 1; // file version: EV_CURRENT
    ident[7] = 0; // OS/ABI identification: System V
    ident[8] = 0; // ABI version: System V third edition

    // padding
    ident[9] = 0;
    ident[10] = 0;
    ident[11] = 0;
    ident[12] = 0;
    ident[13] = 0;
    ident[14] = 0;
    ident[15] = 0;
    return ident;
}

fn create_program() -> Vec<u8> {
    return vec![
        0x66, 0xb8, 0x3c, 0x00, // mov $60, %ax
        0x0f, 0x05, // syscall
    ];
}

fn program_offset() -> u64 {
    (elf64_hdr::SIZE.unwrap() + elf64_phdr::SIZE.unwrap()) as u64
}
const VADDR: u64 = 0x400000;

fn set_elf64_phdr(storage: &mut [u8], program_size: u64) {
    let mut view = elf64_phdr::View::new(storage);
    view._type_mut().write(1); // PT_LOAD
    view.flags_mut().write(0x1 | 0x2 | 0x4); // PF_X | PF_W | PF_R

    // location of segment in file
    view.offset_mut().write(program_offset());
    // virtual address of segment
    view.vaddr_mut().write(VADDR + program_offset());

    view.filesz_mut().write(program_size);
    view.memsz_mut().write(program_size);
    view.align_mut().write(4096);
}

fn main() -> std::io::Result<()> {
    let program = create_program();
    let hdr_sz = elf64_hdr::SIZE.unwrap();
    let phdr_sz = elf64_phdr::SIZE.unwrap();
    let mut buf = vec![0u8; hdr_sz + phdr_sz + program.len()];
    set_elf64_hdr(&mut buf[..hdr_sz]);
    set_elf64_phdr(&mut buf[hdr_sz..hdr_sz + phdr_sz], program.len() as u64);
    buf[program_offset() as usize..].copy_from_slice(&program);

    let mut options = OpenOptions::new();
    options.write(true).create(true).mode(0o755);
    let mut file = options.open("tiny")?;
    file.write_all(&buf)?;
    Ok(())
}
