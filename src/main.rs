extern crate sdl2;
mod cpu;
mod registers;
mod mmu;
use cpu::CPU;


fn main() {
    let mut cpu = CPU::new();
    cpu.registers.h = 1;
    println!("{}", cpu.exec_opcode(0b00100110));
    println!("{}", cpu.registers.h);
}