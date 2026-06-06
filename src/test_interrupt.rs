use super::{
    interrupt, rti, service_pending_interrupt, C64Memory, Flags, Interrupt, Mos6510,
};

fn cpu_at(pc: u16) -> Mos6510 {
    let mut cpu = super::get_cpu();
    cpu.program_counter = pc;
    cpu.stack_pointer = 0xFF;
    cpu
}

#[test]
fn irq_vectors_through_fffe_and_pushes_current_pc() {
    let mut mem = C64Memory::get_empty_mem();
    mem.ram[0xFFFE] = 0x78;
    mem.ram[0xFFFF] = 0x56;
    let cpu = cpu_at(0x1234);

    let res = interrupt(&mut mem, cpu, Interrupt::Irq, false);

    assert_eq!(res.program_counter, 0x5678);
    assert_eq!(res.stack_pointer, 0xFC);
    assert_eq!(mem.ram[0x01FF], 0x12);
    assert_eq!(mem.ram[0x01FE], 0x34);
    assert_eq!(mem.ram[0x01FD], Flags::ALWAYS.bits());
    assert!(res.processor_status.contains(Flags::I_FLAG));
    assert!(!res.processor_status.contains(Flags::B_FLAG));
    assert_eq!(res.cycles_count, 7);
}

#[test]
fn nmi_vectors_through_fffa_even_when_irq_disabled() {
    let mut mem = C64Memory::get_empty_mem();
    mem.ram[0xFFFA] = 0x00;
    mem.ram[0xFFFB] = 0xC0;
    let mut cpu = cpu_at(0x2000);
    cpu.processor_status |= Flags::I_FLAG;

    let res = interrupt(&mut mem, cpu, Interrupt::Nmi, false);

    assert_eq!(res.program_counter, 0xC000);
    assert_eq!(mem.ram[0x01FE], 0x00);
    assert_eq!(mem.ram[0x01FD], (Flags::ALWAYS | Flags::I_FLAG).bits());
}

#[test]
fn pending_irq_is_ignored_when_interrupt_disable_flag_is_set() {
    let mut mem = C64Memory::get_empty_mem();
    mem.cia1.write_byte(0xDC04, 0x01);
    mem.cia1.write_byte(0xDC05, 0x00);
    mem.cia1.write_byte(0xDC0D, 0x81);
    mem.cia1.write_byte(0xDC0E, 0x11);
    mem.tick(1);
    let mut cpu = cpu_at(0x2000);
    cpu.processor_status |= Flags::I_FLAG;

    let res = service_pending_interrupt(&mut mem, cpu);

    assert!(!res.1);
    assert_eq!(res.0.program_counter, 0x2000);
}

#[test]
fn pending_irq_is_serviced_when_interrupt_disable_flag_is_clear() {
    let mut mem = C64Memory::get_empty_mem();
    mem.ram[0xFFFE] = 0x00;
    mem.ram[0xFFFF] = 0xEA;
    mem.cia1.write_byte(0xDC04, 0x01);
    mem.cia1.write_byte(0xDC05, 0x00);
    mem.cia1.write_byte(0xDC0D, 0x81);
    mem.cia1.write_byte(0xDC0E, 0x11);
    mem.tick(1);
    let cpu = cpu_at(0x2000);

    let res = service_pending_interrupt(&mut mem, cpu);

    assert!(res.1);
    assert_eq!(res.0.program_counter, 0xEA00);
    assert!(mem.irq_pending());
    assert_eq!(mem.cia1.read_byte(0xDC0D), 0x81);
    assert!(!mem.irq_pending());
}

#[test]
fn rti_restores_status_and_program_counter_from_stack() {
    let mut mem = C64Memory::get_empty_mem();
    mem.ram[0x01FD] = (Flags::ALWAYS | Flags::C_FLAG).bits();
    mem.ram[0x01FE] = 0x34;
    mem.ram[0x01FF] = 0x12;
    let mut cpu = cpu_at(0xEA31);
    cpu.stack_pointer = 0xFC;
    cpu.processor_status = Flags::ALWAYS | Flags::I_FLAG;

    let res = rti(&mut mem, cpu);

    assert_eq!(res.program_counter, 0x1234);
    assert_eq!(res.stack_pointer, 0xFF);
    assert_eq!(res.processor_status, Flags::ALWAYS | Flags::C_FLAG);
    assert_eq!(res.cycles_count, 6);
}
