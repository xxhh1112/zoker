use crypto::digest::Digest;
use crypto::sha2::Sha256;

macro_rules! get_bit {
    ($x: expr, $i: expr) => {{
        ($x >> $i) & 0x01
    }};
}

macro_rules! set_bit {
    ($x: expr, $i: expr, $b: expr) => {{
        $x = if $b & 1 != 0 {
            $x | (1 << $i)
        } else {
            $x & !(1 << $i)
        };
    }};
}

pub type IKosResult<T> = Result<T, IKosError>;

#[derive(Debug, PartialEq)]
pub struct IKosError {
    pub error: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IKosView {
    rand_tape_seed: Vec<u8>,
    in_data: Vec<u8>,
    out_data32: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IKosContext {
    ikos_view: IKosView,
    randomness: Vec<u32>,
    used_rand_ctr: u32,
    out_view_ctr32: u32,
    is_verify_mode: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IKosVariable4P {
    value: Vec<u32>,
    ctx: Vec<IKosContext>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IKosVariable4V {
    value: Vec<u32>,
    ctx: Vec<IKosContext>,
    inst_random: Vec<u32>,
}

fn generate_random(num: usize) -> Vec<u8> {
    // TODO: Use Random
    vec![0; num]
}

fn generate_all_randomness(rand_len: usize) -> Vec<u32> {
    // TODO: Use Random
    let len = rand_len / 32;
    vec![0; len]
}

fn get_next_random_from_context(ctx: &mut IKosContext) -> IKosResult<u32> {
    if ctx.randomness.len() as u32 <= ctx.used_rand_ctr {
        return Err(IKosError {
            error: String::from("All pre-generated randomness are exhausted!"),
        });
    }
    let rand = ctx.randomness[ctx.used_rand_ctr as usize];
    ctx.used_rand_ctr += 1;
    Ok(rand)
}

impl IKosView {
    pub fn new() -> Self {
        let ozkb_rand_tape_seed_len = 16;
        IKosView {
            rand_tape_seed: generate_random(ozkb_rand_tape_seed_len),
            in_data: vec![],
            out_data32: vec![],
        }
    }
}

impl IKosContext {
    pub fn new(rand_tape_len: usize, is_verify_mode: bool) -> Self {
        IKosContext {
            ikos_view: IKosView::new(),
            randomness: generate_all_randomness(rand_tape_len * 8),
            used_rand_ctr: 0,
            out_view_ctr32: 0,
            is_verify_mode,
        }
    }

    fn commit_ikos_context(&mut self) -> Vec<u8> {
        let mut sha = Sha256::new();
        sha.input(&self.ikos_view.rand_tape_seed);
        if !self.ikos_view.out_data32.is_empty() {
            sha.input(unsafe {
                std::slice::from_raw_parts(
                    self.ikos_view.out_data32.as_ptr() as *const u8,
                    &self.ikos_view.out_data32.len() * std::mem::size_of::<i32>(),
                )
            });
        }
        sha.result_str().as_bytes().to_vec()
    }

    fn str_to_ikos_view(view_part1: String, view_part2: String) {
        let view = IKosView::new();
        // TODO: 미구현
    }

    fn ikos_view_str(view: &IKosView) {
        // TODO: 미구현
    }

    fn dump_ikos_view(view: &IKosView) {
        // 안 쓰는듯
    }
}

impl IKosVariable4P {
    pub fn new() -> Self {
        IKosVariable4P {
            value: vec![0; 3],
            ctx: vec![],
        }
    }

    pub fn new_value(value: u32) -> Self {
        IKosVariable4P {
            value: vec![value; 3],
            ctx: vec![],
        }
    }

    pub fn new_share(value: Vec<u32>, ctx: Vec<IKosContext>) -> Self {
        IKosVariable4P { value, ctx }
    }

    fn is_empty_context(&self) -> bool {
        self.ctx.is_empty()
    }

    fn copy_context(&mut self, rhs_ctx: Vec<IKosContext>) {
        self.ctx = rhs_ctx;
    }

    pub fn negate(mut self) -> Self {
        for i in 0..3 {
            self.value[i] = !self.value[i];
        }
        self
    }

    pub fn or(mut self, rhs: &IKosVariable4P) -> Self {
        if self.is_empty_context() {
            self.copy_context(rhs.ctx.clone());
        }
        for i in 0..3 {
            self.value[i] ^= rhs.value[i];
        }
        self
    }

    pub fn rshift(mut self, n: u32) -> Self {
        for i in 0..3 {
            self.value[i] >>= n;
        }
        self
    }

    pub fn lshift(mut self, n: u32) -> Self {
        for i in 0..3 {
            self.value[i] <<= n;
        }
        self
    }

    pub fn bit_and(mut self, rhs: &IKosVariable4P) -> Self {
        let mut rand = vec![0; 3];
        let mut out = vec![0; 3];

        if self.is_empty_context() && rhs.is_empty_context() {
            for i in 0..3 {
                self.value[i] &= rhs.value[i];
            }
            return self;
        }

        if self.is_empty_context() {
            self.copy_context(rhs.ctx.clone());
        }

        for i in 0..3 {
            rand[i] = get_next_random_from_context(&mut self.ctx[i]).unwrap();
        }
        for i in 0..3 {
            out[i] = (self.value[i] & rhs.value[(i + 1) % 3])
                ^ (self.value[(i + 1) % 3] & rhs.value[i])
                ^ (self.value[i] & rhs.value[i])
                ^ rand[i]
                ^ rand[(i + 1) % 3];
        }
        for i in 0..3 {
            self.value[i] = out[i];
            self.ctx[i].ikos_view.out_data32.push(self.value[i]);
        }
        self
    }

    pub fn bit_or(mut self, rhs: &IKosVariable4P) -> Self {
        if self.is_empty_context() {
            self.copy_context(rhs.ctx.clone());
        }
        for i in 0..3 {
            self.value[i] |= rhs.value[i];
        }
        self
    }

    pub fn add(mut self, rhs: &IKosVariable4P) -> Self {
        let mut a = vec![0; 3];
        let mut b = vec![0; 3];
        let mut c = 0;
        let mut rand = vec![0; 3];
        let mut out = vec![0; 3];

        if self.is_empty_context() && rhs.is_empty_context() {
            for i in 0..3 {
                self.value[i] += rhs.value[i];
            }
            return self;
        }

        if self.is_empty_context() {
            self.copy_context(rhs.ctx.clone());
        }
        for i in 0..3 {
            rand[i] |= get_next_random_from_context(&mut self.ctx[i]).unwrap();
        }

        for i in 0..31 {
            for j in 0..3 {
                a[j] = get_bit!(self.value[j] ^ out[j], i);
                b[j] = get_bit!(rhs.value[j] ^ out[j], i);
            }
            for j in 0..3 {
                c = (a[j] & b[(j + 1) % 3])
                    ^ (a[(j + 1) % 3] & b[j])
                    ^ get_bit!(rand[(j + 1) % 3], i);
                set_bit!(
                    out[j],
                    i + 1,
                    (c ^ (a[j] & b[j]) ^ (get_bit!(out[j], i)) ^ (get_bit!(rand[j], i)))
                );
            }
        }

        for i in 0..3 {
            self.value[i] = self.value[i] ^ rhs.value[i] ^ out[i];
            self.ctx[i].ikos_view.out_data32.push(out[i]);
        }
        self
    }
}

impl IKosVariable4V {
    pub fn new() -> Self {
        IKosVariable4V {
            value: vec![0; 3],
            ctx: vec![],
            inst_random: vec![],
        }
    }

    pub fn new_value(value: u32) -> Self {
        IKosVariable4V {
            value: vec![value; 3],
            ctx: vec![],
            inst_random: vec![],
        }
    }

    pub fn new_share(value: Vec<u32>, ctx: Vec<IKosContext>) -> Self {
        IKosVariable4V {
            value,
            ctx,
            inst_random: vec![],
        }
    }

    pub fn new_share_with_random(
        value: Vec<u32>,
        ctx: Vec<IKosContext>,
        inst_random: Vec<u32>,
    ) -> Self {
        IKosVariable4V {
            value,
            ctx,
            inst_random,
        }
    }

    // TODO: 이 함수들은 P 에서도 사용함
    fn is_empty_context(&self) -> bool {
        self.ctx.is_empty()
    }

    // TODO: 이 함수들은 P 에서도 사용함
    fn copy_context(&mut self, rhs_ctx: Vec<IKosContext>) {
        self.ctx = rhs_ctx;
    }

    pub fn negate(mut self) -> Self {
        for i in 0..3 {
            self.value[i] = !self.value[i];
        }
        self
    }

    pub fn or(mut self, rhs: &IKosVariable4V) -> Self {
        if self.is_empty_context() {
            self.copy_context(rhs.ctx.clone());
        }
        for i in 0..3 {
            self.value[i] ^= rhs.value[i];
        }
        self
    }

    pub fn rshift(mut self, n: u32) -> Self {
        for i in 0..3 {
            self.value[i] >>= n;
        }
        self
    }

    pub fn lshift(mut self, n: u32) -> Self {
        for i in 0..3 {
            self.value[i] <<= n;
        }
        self
    }

    fn require_reconstruct(ctx: &Vec<IKosContext>) -> bool {
        ctx[0].ikos_view.out_data32.len() == ctx[1].ikos_view.out_data32.len()
    }

    fn get_next_random(&mut self, i: usize) -> IKosResult<u32> {
        if i < 2 {
            get_next_random_from_context(&mut self.ctx[i])
        } else {
            Ok(0)
        }
    }

    pub fn bit_and(mut self, rhs: &IKosVariable4V) -> IKosResult<Self> {
        let mut rand = vec![0; 3];
        let mut out = 0;

        if self.is_empty_context() && rhs.is_empty_context() {
            for i in 0..3 {
                self.value[i] &= rhs.value[i];
            }
            return Ok(self);
        }

        if self.is_empty_context() {
            self.copy_context(rhs.ctx.clone());
        }

        for i in 0..3 {
            rand[i] = get_next_random_from_context(&mut self.ctx[i]).unwrap();
        }

        out = (self.value[0] & rhs.value[1])
            ^ (self.value[1] & rhs.value[0])
            ^ (self.value[0] & rhs.value[0])
            ^ rand[0]
            ^ rand[1];

        if self.ctx[0].is_verify_mode {
            // verify mode
            if !IKosVariable4V::require_reconstruct(&self.ctx) {
                if out != self.ctx[0].ikos_view.out_data32[self.ctx[0].out_view_ctr32] {
                    return Err(IKosError {
                        error: String::from("_IkosVariable4V & operation fail."),
                    });
                }
            } else {
                self.ctx[0].ikos_view.out_data32.push(out);
            }
            self.value[0] = out;
            self.value[1] = self.ctx[1].ikos_view.out_data32[self.ctx[1].out_view_ctr32];
            for i in 0..3 {
                self.ctx[i].out_view_ctr32 += 1;
            }
        } else {
            // Non verify mode
            self.value[0] = out;
            self.value[1] = self.inst_random[self.inst_random[0]];
            self.inst_random[0] += 1;

            for i in 0..3 {
                self.ctx[i].ikos_view.out_data32.push(self.value[i]);
            }
        }

        Ok(self)
    }

    pub fn bit_or(mut self, rhs: &IKosVariable4V) -> Self {
        if self.is_empty_context() {
            self.copy_context(rhs.ctx.clone());
        }
        for i in 0..3 {
            self.value[i] |= rhs.value[i];
        }
        self
    }

    pub fn add(mut self, rhs: &IKosVariable4V) -> IKosResult<Self> {
        let mut a = vec![0; 3];
        let mut b = vec![0; 3];
        let mut c = 0;
        let mut rand = vec![0; 3];
        let mut out = vec![0; 3];

        if self.is_empty_context() && rhs.is_empty_context() {
            for i in 0..3 {
                self.value[i] += rhs.value[i];
            }
            return Ok(self);
        }

        if self.is_empty_context() {
            self.copy_context(rhs.ctx.clone());
        }
        for i in 0..3 {
            rand[i] = self.get_next_random(i)?;
        }

        if self.ctx[0].is_verify_mode {
            // verify mode
            let required = IKosVariable4V::require_reconstruct(&self.ctx);
            for i in 0..3 {
                if !required || i != 0 {
                    out[i] = self.ctx[i].ikos_view.out_data32[self.ctx[i].out_view_ctr32];
                }
                self.ctx[i].out_view_ctr32 += 1;
            }
            for i in 0..31 {
                for j in 0..3 {
                    a[j] = get_bit!(self.value[j] ^ out[j], i);
                    b[j] = get_bit!(rhs.value[j] ^ out[j], i);
                }
                c = (a[0] & b[1]) ^ (a[1] & b[0]) ^ (get_bit!(rand[1], i));
                if !required {
                    if (c ^ (a[0] & b[0]) ^ (get_bit!(out[0], i)) ^ (get_bit!(rand[0], i)))
                        != (get_bit!(out[0], i + 1))
                    {
                        return Err(IKosError {
                            error: String::from("_IkosVariable4V + operation fail."),
                        });
                    }
                } else {
                    set_bit!(
                        out[0],
                        i + 1,
                        c ^ (a[0] & b[0]) ^ (get_bit!(out[0], i)) ^ (get_bit!(rand[0], i))
                    );
                }
            }
            if required {
                self.ctx[0].ikos_view.out_data32.push(out[0]);
            }
            for i in 0..3 {
                self.value[i] = self.value[i] ^ rhs.value[i] ^ out[i];
            }
        } else {
            // Non verify mode
            out[1] = self.inst_random[self.inst_random[0]];
            self.inst_random[0] += 1;
            set_bit!(out[1], 0, 0);

            for i in 0..31 {
                for j in 0..3 {
                    a[j] = get_bit!(self.value[j] ^ out[j], i);
                    b[j] = get_bit!(rhs.value[j] ^ out[j], i);
                }
                c = (a[0] & b[1]) ^ (a[1] & b[0]) ^ (get_bit!(rand[1], i));
                set_bit!(
                    out[0],
                    i + 1,
                    c ^ (a[0] & b[0]) ^ (get_bit!(out[0], i)) ^ (get_bit!(rand[0], i))
                );
            }

            for i in 0..3 {
                self.value[i] = self.value[i] ^ rhs.value[i] ^ out[i];
                self.ctx[i].ikos_view.out_data32.push(out[i]);
            }
        }
        Ok(self)
    }
}
