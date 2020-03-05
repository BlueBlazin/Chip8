mod utils;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// -----------------------------------------------
// CHIP-8
// A Chip 8 emulator written in Rust.
// -----------------------------------------------
use rand::random;
use wasm_bindgen::prelude::*;

const MEM_OFFSET: u16 = 0x200;

const FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

#[wasm_bindgen]
pub struct Vm {
    memory: [u8; 4096],
    v: [u8; 16],
    i: u16,
    pc: u16,
    gfx: Vec<u8>,
    delay_timer: u8,
    sound_timer: u8,
    stack: [u16; 16],
    sp: u16,
    key: [u8; 16],
    drawflag: bool,
}

#[wasm_bindgen]
impl Vm {
    pub fn new() -> Self {
        let mut memory = [0; 4096];
        for i in 0..80 {
            memory[i] = FONTSET[i];
        }

        Vm {
            i: 0x0,
            sp: 0x0,
            pc: MEM_OFFSET,
            memory,
            gfx: vec![0; 64 * 32],
            stack: [0; 16],
            v: [0; 16],
            key: [0; 16],
            delay_timer: 0x0,
            sound_timer: 0x0,
            drawflag: true,
        }
    }

    pub fn load(&mut self, program: Vec<u8>) {
        let offset = MEM_OFFSET as usize;
        for (i, x) in program.into_iter().enumerate() {
            self.memory[offset + i] = x;
        }
    }

    pub fn tick(&mut self) {
        let pc = self.pc as usize;
        let opcode = (self.memory[pc] as u16) << 8 | (self.memory[pc + 1] as u16);
        self.drawflag = false;

        // match opcodes and perform operations
        match opcode & 0xF000 {
            0x0000 => match opcode & 0x000F {
                0x0000 => self.disp_clear(),
                0x000E => self.subroutine_ret(),
                _ => panic!("Invalid Opcode"),
            },
            0x1000 => self.jmp(opcode & 0x0FFF),
            0x2000 => self.subroutine_call(opcode & 0x0FFF),
            0x3000 => self.skip_eq_imm(((opcode & 0x0F00) >> 8) as usize, (opcode & 0x00FF) as u8),
            0x4000 => self.skip_neq_imm(((opcode & 0x0F00) >> 8) as usize, (opcode & 0x00FF) as u8),
            0x5000 => self.skip_eq(
                ((opcode & 0x0F00) >> 8) as usize,
                ((opcode & 0x00F0) >> 4) as usize,
            ),
            0x6000 => self.set_imm(((opcode & 0x0F00) >> 8) as usize, (opcode & 0x00FF) as u8),
            0x7000 => self.add_imm(((opcode & 0x0F00) >> 8) as usize, (opcode & 0x00FF) as u8),
            0x8000 => match opcode & 0x000F {
                0x0000 => self.set(
                    ((opcode & 0x0F00) >> 8) as usize,
                    ((opcode & 0x00F0) >> 4) as usize,
                ),
                0x0001 => self.orr(
                    ((opcode & 0x0F00) >> 8) as usize,
                    ((opcode & 0x00F0) >> 4) as usize,
                ),
                0x0002 => self.and(
                    ((opcode & 0x0F00) >> 8) as usize,
                    ((opcode & 0x00F0) >> 4) as usize,
                ),
                0x0003 => self.xor(
                    ((opcode & 0x0F00) >> 8) as usize,
                    ((opcode & 0x00F0) >> 4) as usize,
                ),
                0x0004 => self.add(
                    ((opcode & 0x0F00) >> 8) as usize,
                    ((opcode & 0x00F0) >> 4) as usize,
                ),
                0x0005 => self.sub(
                    ((opcode & 0x0F00) >> 8) as usize,
                    ((opcode & 0x00F0) >> 4) as usize,
                ),
                0x0006 => self.rshift(((opcode & 0x0F00) >> 8) as usize),
                0x0007 => self.inv_sub(
                    ((opcode & 0x0F00) >> 8) as usize,
                    ((opcode & 0x00F0) >> 4) as usize,
                ),
                0x000E => self.lshift(((opcode & 0x0F00) >> 8) as usize),
                _ => panic!("Invalid Opcode"),
            },
            0x9000 => self.skip_neq(
                ((opcode & 0x0F00) >> 8) as usize,
                ((opcode & 0x00F0) >> 4) as usize,
            ),
            0xA000 => self.iset(opcode & 0x0FFF),
            0xB000 => self.jmp_offset(opcode & 0x0FFF),
            0xC000 => self.rand_imm(((opcode & 0x0F00) >> 8) as usize, (opcode & 0x00FF) as u8),
            0xD000 => self.draw_sprite(
                ((opcode & 0x0F00) >> 8) as usize,
                ((opcode & 0x00F0) >> 4) as usize,
                (opcode & 0x000F) as usize,
            ),
            0xE000 => match opcode & 0x00FF {
                0x009E => self.skip_eq_key(((opcode & 0x0F00) >> 8) as usize),
                0x00A1 => self.skip_neq_key(((opcode & 0x0F00) >> 8) as usize),
                _ => panic!("Invalid Opcode"),
            },
            0xF000 => match opcode & 0x00FF {
                0x0007 => self.get_delay(((opcode & 0x0F00) >> 8) as usize),
                0x000A => {
                    let waiting = self.get_key(((opcode & 0x0F00) >> 8) as usize);
                    if waiting {
                        return;
                    }
                }
                0x0015 => self.set_delay(((opcode & 0x0F00) >> 8) as usize),
                0x0018 => self.set_sound(((opcode & 0x0F00) >> 8) as usize),
                0x001E => self.iadd(((opcode & 0x0F00) >> 8) as usize),
                0x0029 => self.iset_sprite(((opcode & 0x0F00) >> 8) as usize),
                0x0033 => self.set_bcd(((opcode & 0x0F00) >> 8) as usize),
                0x0055 => self.reg_dump(((opcode & 0x0F00) >> 8) as usize),
                0x0065 => self.reg_load(((opcode & 0x0F00) >> 8) as usize),
                _ => panic!("Invalid Opcode"),
            },
            _ => panic!("Invalid Opcode"),
        }
    }

    #[wasm_bindgen(js_name = updateTimers)]
    pub fn update_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                // do something
            }
            self.sound_timer -= 1;
        }
    }

    #[wasm_bindgen(js_name = keyDown)]
    pub fn key_down(&mut self, idx: usize) {
        self.key[idx] = 1;
    }

    #[wasm_bindgen(js_name = keyUp)]
    pub fn key_up(&mut self, idx: usize) {
        self.key[idx] = 0;
    }

    pub fn screen(&self) -> *const u8 {
        self.gfx.as_ptr()
    }

    #[wasm_bindgen(js_name = drawFlag)]
    pub fn draw_flag(&self) -> bool {
        self.drawflag
    }

    // -----------------------------------------------
    // Instruction Set
    // -----------------------------------------------

    // 00E0	Display	disp_clear()	Clears the screen.
    fn disp_clear(&mut self) {
        self.gfx = vec![0; 64 * 32];
        self.drawflag = true;
        self.pc += 2;
    }

    // 00EE	Flow	return;	Returns from a subroutine.
    fn subroutine_ret(&mut self) {
        self.sp -= 1;
        let pc = self.stack[self.sp as usize];
        self.pc = pc;
    }

    // 1NNN	Flow	goto NNN;	Jumps to address NNN.
    fn jmp(&mut self, addr: u16) {
        self.pc = addr;
    }

    // 2NNN	Flow	*(0xNNN)()	Calls subroutine at NNN.
    fn subroutine_call(&mut self, addr: u16) {
        let sp = self.sp as usize;
        // set return address to next instruction
        self.stack[sp] = self.pc + 2;
        self.sp += 1;
        self.pc = addr;
    }

    // 3XNN	Cond	if(Vx==NN)	Skips the next instruction if VX equals NN. (Usually the next instruction is a jump to skip a code block)
    fn skip_eq_imm(&mut self, x: usize, right: u8) {
        self.pc += if self.v[x] == right { 4 } else { 2 };
    }

    // 4XNN	Cond	if(Vx!=NN)	Skips the next instruction if VX doesn't equal NN. (Usually the next instruction is a jump to skip a code block)
    fn skip_neq_imm(&mut self, x: usize, right: u8) {
        self.pc += if self.v[x] != right { 4 } else { 2 };
    }

    // 5XY0	Cond	if(Vx==Vy)	Skips the next instruction if VX equals VY. (Usually the next instruction is a jump to skip a code block)
    fn skip_eq(&mut self, x: usize, y: usize) {
        self.pc += if self.v[x] == self.v[y] { 4 } else { 2 };
    }

    // 6XNN	Const	Vx = NN	Sets VX to NN.
    fn set_imm(&mut self, x: usize, value: u8) {
        self.v[x] = value;
        self.pc += 2;
    }

    // 7XNN	Const	Vx += NN	Adds NN to VX. (Carry flag is not changed)
    fn add_imm(&mut self, x: usize, value: u8) {
        let (sum, _) = self.v[x].overflowing_add(value);
        self.v[x] = sum;
        self.pc += 2;
    }

    // 8XY0	Assign	Vx=Vy	Sets VX to the value of VY.
    fn set(&mut self, x: usize, y: usize) {
        self.v[x] = self.v[y];
        self.pc += 2;
    }

    // 8XY1	BitOp	Vx=Vx|Vy	Sets VX to VX or VY. (Bitwise OR operation)
    fn orr(&mut self, x: usize, y: usize) {
        self.v[x] = self.v[x] | self.v[y];
        self.pc += 2;
    }

    // 8XY2	BitOp	Vx=Vx&Vy	Sets VX to VX and VY. (Bitwise AND operation)
    fn and(&mut self, x: usize, y: usize) {
        self.v[x] = self.v[x] & self.v[y];
        self.pc += 2;
    }

    // 8XY3[a]	BitOp	Vx=Vx^Vy	Sets VX to VX xor VY.
    fn xor(&mut self, x: usize, y: usize) {
        self.v[x] = self.v[x] ^ self.v[y];
        self.pc += 2;
    }

    // 8XY4	Math	Vx += Vy	Adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there isn't.
    fn add(&mut self, x: usize, y: usize) {
        let (value, overflow) = self.v[x].overflowing_add(self.v[y]);
        self.v[0xF] = if overflow { 1 } else { 0 };
        self.v[x] = value;
        self.pc += 2;
    }

    // 8XY5	Math	Vx -= Vy	VY is subtracted from VX. VF is set to 0 when there's a borrow, and 1 when there isn't.
    fn sub(&mut self, x: usize, y: usize) {
        let (value, borrow) = self.v[x].overflowing_sub(self.v[y]);
        self.v[0xF] = if borrow { 0 } else { 1 };
        self.v[x] = value;
        self.pc += 2;
    }

    // 8XY6[a]	BitOp	Vx>>=1	Stores the least significant bit of VX in VF and then shifts VX to the right by 1.[b]
    fn rshift(&mut self, x: usize) {
        self.v[0xF] = self.v[x] & 0x01;
        self.v[x] = self.v[x] >> 1;
        self.pc += 2;
    }

    // 8XY7[a]	Math	Vx=Vy-Vx	Sets VX to VY minus VX. VF is set to 0 when there's a borrow, and 1 when there isn't.
    fn inv_sub(&mut self, x: usize, y: usize) {
        let (value, borrow) = self.v[y].overflowing_sub(self.v[x]);
        self.v[0xF] = if borrow { 0 } else { 1 };
        self.v[x] = value;
        self.pc += 2;
    }

    // 8XYE[a]	BitOp	Vx<<=1	Stores the most significant bit of VX in VF and then shifts VX to the left by 1.[b]
    fn lshift(&mut self, x: usize) {
        self.v[0xF] = (self.v[x] & 0x80) >> 7;
        self.v[x] = self.v[x] << 1;
        self.pc += 2;
    }

    // 9XY0	Cond	if(Vx!=Vy)	Skips the next instruction if VX doesn't equal VY. (Usually the next instruction is a jump to skip a code block)
    fn skip_neq(&mut self, x: usize, y: usize) {
        self.pc += if self.v[x] != self.v[y] { 4 } else { 2 };
    }

    // ANNN	MEM	I = NNN	Sets I to the address NNN.
    fn iset(&mut self, addr: u16) {
        self.i = addr;
        self.pc += 2;
    }

    // BNNN	Flow	PC=V0+NNN	Jumps to the address NNN plus V0.
    fn jmp_offset(&mut self, addr: u16) {
        self.pc = addr + (self.v[0] as u16);
    }

    // CXNN	Rand	Vx=rand()&NN	Sets VX to the result of a bitwise and operation on a random number (Typically: 0 to 255) and NN.
    fn rand_imm(&mut self, x: usize, value: u8) {
        self.v[x] = value & random::<u8>();
        self.pc += 2;
    }

    // DXYN	Disp	draw(Vx,Vy,N)	Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels and a height of N pixels.
    // Each row of 8 pixels is read as bit-coded starting from memory location I;
    // I value doesn’t change after the execution of this instruction.
    // As described above, VF is set to 1 if any screen pixels are flipped from set to unset when the sprite is drawn, and to 0 if that doesn’t happen
    fn draw_sprite(&mut self, x: usize, y: usize, n: usize) {
        let col_offset = self.v[x] as usize;
        let row_offset = self.v[y] as usize;
        let i = self.i as usize;
        self.v[0xF] = 0;
        for row in 0..n {
            let pixel = self.memory[i + row];
            for col in 0..8 {
                let bit = (0x80 >> col) & pixel;
                if bit != 0 {
                    let idx = (row_offset + row) * 64 + (col_offset + col);
                    if self.gfx[idx] == 1 {
                        self.v[0xF] = 1;
                    }
                    self.gfx[idx] = self.gfx[idx] ^ 1;
                }
            }
        }
        self.drawflag = true;
        self.pc += 2;
    }

    // EX9E	KeyOp	if(key()==Vx)	Skips the next instruction if the key stored in VX is pressed. (Usually the next instruction is a jump to skip a code block)
    fn skip_eq_key(&mut self, x: usize) {
        println!("skip_eq_key called with {}, {}", x, self.v[x]);
        let idx = self.v[x] as usize;
        self.pc += if self.key[idx] != 0 { 4 } else { 2 };
    }

    // EXA1	KeyOp	if(key()!=Vx)	Skips the next instruction if the key stored in VX isn't pressed. (Usually the next instruction is a jump to skip a code block)
    fn skip_neq_key(&mut self, x: usize) {
        println!("skip_neq_key called with {}, {}", x, self.v[x]);
        let idx = self.v[x] as usize;
        self.pc += if self.key[idx] == 0 { 4 } else { 2 };
    }

    // FX07	Timer	Vx = get_delay()	Sets VX to the value of the delay timer.
    fn get_delay(&mut self, x: usize) {
        self.v[x] = self.delay_timer;
        self.pc += 2;
    }

    // FX0A	KeyOp	Vx = get_key()	A key press is awaited, and then stored in VX. (Blocking Operation. All instruction halted until next key event)
    fn get_key(&mut self, x: usize) -> bool {
        println!("get_key called with {}, {}", x, self.v[x]);
        for idx in 0..16 {
            if self.key[idx] != 0 {
                println!("got key");
                self.v[x] = idx as u8;
                self.pc += 2;
                return false;
            }
        }
        true
    }

    // FX15	Timer	delay_timer(Vx)	Sets the delay timer to VX.
    fn set_delay(&mut self, x: usize) {
        self.delay_timer = self.v[x];
        self.pc += 2;
    }

    // FX18	Sound	sound_timer(Vx)	Sets the sound timer to VX.
    fn set_sound(&mut self, x: usize) {
        self.sound_timer = self.v[x];
        self.pc += 2;
    }

    // FX1E	MEM	I +=Vx	Adds VX to I. VF is set to 1 when there is a range overflow (I+VX>0xFFF), and to 0 when there isn't.[c]
    fn iadd(&mut self, x: usize) {
        let (value, mut overflow) = self.i.overflowing_add(self.v[x] as u16);
        overflow = overflow || value > 0xFFF;
        self.v[0xF] = if overflow { 1 } else { 0 };
        self.i = value;
        self.pc += 2;
    }

    // FX29	MEM	I=sprite_addr[Vx]	Sets I to the location of the sprite for the character in VX. Characters 0-F (in hexadecimal) are represented by a 4x5 font.
    fn iset_sprite(&mut self, x: usize) {
        self.i = (self.v[x] as u16) * 0x5;
        self.pc += 2;
    }

    // FX33	BCD	set_BCD(Vx);
    // Stores the binary-coded decimal representation of VX, with the most significant of three digits at the address in I, the middle digit at I plus 1, and the least significant digit at I plus 2.
    // (In other words, take the decimal representation of VX, place the hundreds digit in memory at location in I, the tens digit at location I+1, and the ones digit at location I+2.)
    fn set_bcd(&mut self, x: usize) {
        let i = self.i as usize;
        self.memory[i] = self.v[x] / 100;
        self.memory[i + 1] = (self.v[x] / 10) % 10;
        self.memory[i + 2] = (self.v[x] % 100) % 10;
        self.pc += 2;
    }

    // FX55	MEM	reg_dump(Vx,&I)	Stores V0 to VX (including VX) in memory starting at address I. The offset from I is increased by 1 for each value written, but I itself is left unmodified.[d]
    fn reg_dump(&mut self, x: usize) {
        let i = self.i as usize;
        for idx in 0..x + 1 {
            self.memory[i + idx] = self.v[idx];
        }
        self.i = ((i + x) as u16) + 1;
        self.pc += 2;
    }

    // FX65	MEM	reg_load(Vx,&I)	Fills V0 to VX (including VX) with values from memory starting at address I. The offset from I is increased by 1 for each value written, but I itself is left unmodified.[d]
    fn reg_load(&mut self, x: usize) {
        let i = self.i as usize;
        for idx in 0..x + 1 {
            self.v[idx] = self.memory[i + idx];
        }
        self.i = ((i + x) as u16) + 1;
        self.pc += 2;
    }
}
