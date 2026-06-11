/* ACCT-RS
 *
 * BSD Processing for RLUX-OS.
 *
 * Author: Allexander Bergmans <al@libregrad.pp.ua>
 *
 * Some code based on the source of:
 * https://github.com/torvalds/linux/blob/master/kernel/acct.c
 *
 * This file implements a BSD-style process accounting. Whenever
 * a process exists, an accounting record of type ´struct acct´ is
 * writen to the file specified with the acct() system call.
 *
 * (C) Copyright 2025-2026 - Allexander Bergmans - Libregrad.
 *
 * 
 */

// =========================================================
#![allow(non_camel_case_types)]
// ==========================================================
// The following imports are like 
// linux/slab.h. This is inspired
// by the Linux Kernel source code
// located @:
// https://github.com/torvalds/linux/blob/master/kernel/acct.c
// ===========================================================
use kernel::alloc::flags;
use kernel::alloc:KBox;
use kernel::uapi::acct;
// ===========================================================
// comp_t is a 16-bit "floating" point number with a 3-bit
// base-8 exponent and a 13-bit fraction.
pub type comp_t = u16;
pub type comp2_t = u32;
// ===========================================================
// Accounting Flags
// The actual accounting
// flags.
pub const AFORK: u8     = 0x01; // Executed fork, but no exec.
pub const ASU: u8       = 0x02; // Used super-user privilages.
pub const ACOMPAT: u8   = 0x04; // Used compatibility mode.
pub const ACORE: u8     = 0x08; // Dumped core
pub const AXSIG: u8     = 0x10; // Killed by a signal
pub const AGROUP: u8    = 0x20; // Was the last task of the process
                                // group.

static ACCT_PARAM: [i32; 3] = [4, 2, 30];

pub const ACCT_COMM: usize = 16;
pub const RESUME: i32  =  ACCT_PARAM[0];     // free space - resume
pub const SUSPEND: i32 =  ACCT_PARAM[1];     // free space - suspend
pub const ACCT_TIMEOUT: i32 = ACCT_PARAM[2]; // Second timeout between checks

