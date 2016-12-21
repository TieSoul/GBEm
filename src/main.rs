extern crate sdl2;
mod cpu;
mod registers;
use cpu::CPU;


fn main() {
    let mut cpu = CPU::new();
    let r1 = cpu.alu_add(128, 127);
    let r2 = cpu.alu_adc(128,128);
    let r3 = cpu.alu_adc(128,0);
    println!("{}", r1);
    println!("{}", r2);
    println!("{}", r3);
}