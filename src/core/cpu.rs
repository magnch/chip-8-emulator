use super::display::Display;
use super::memory::Memory;

struct Cpu {
    ram: Memory,
    display: Display,
}