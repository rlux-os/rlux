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
use core::sync::atomic::AtomicI64;
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

// Assuming acct_t is a typedef, for a specific
// account layout.
pub type AcctT = u32;

pub const ACCT_COMM: usize = 16;
pub const RESUME: i32  =  ACCT_PARAM[0];     // free space - resume
pub const SUSPEND: i32 =  ACCT_PARAM[1];     // free space - suspend
pub const ACCT_TIMEOUT: i32 = ACCT_PARAM[2]; // Second timeout between checks

pub struct BsdAcctStruct {
    pub pin: FsPin,             // pin: In C, this hooks into the file system in system,
                                // to provent unmounting. For now, we stub it out or rep-
                                // resent it as a pointer/custom type.
    pub count: AtomicI64,       // count: atomic_long_t handles thread-safe reference counting,
                                // on 64-bit architectures, atomic_long_t maps directly to 
                                // AtomicI64.
    pub rcu: RcuHead,           // rcu: Read-Copy Update reference head used for deferred, safe
                                // memory deletion
    pub lock: Mutex,            // lock: A standard kernel mutex lock to protect this struct
                                // accross
                                // threads.
    pub active: bool,           // Booleans that translate directly to Rust's primitive bool.
    pub check_space: bool,      // ^^^^
    pub needcheck: usize;       // needcheck: unsigned long translates to usize.
    pub file *mut File,         // Raw pointer to a file structure and a PID namespace
                                // struct. In Rust, pointers to kenrel objects are usually
                                // wrapper in safe types. But for low level compatibility,
                                // we can use raw pointers.
    pub ns *mut PidNamespace,   // ^^^^
    pub work: WorkStruct,       // work: An asynchronous worker item to check disk space 
                                // in the background.
    pub done: Completion,       // done: A synchronization primitive used to wait for a 
                                // task to complete.
    pub ac: crate::acct::acct,  // ac: The actual acct data structure we translated earlier
}

impl BsdAcctStruct {

    // Fills the accounting structure
    // with process info.
    fn fill_ac(&mut self) {
        let cur_task = current();
        let pacct = &(*cur_task.signal).pacct;
        let file = match self.file {
            Some(f) => &*f,
            None => return,
        };
        let ac = &mut self.ac;

        // Active state checks
        if timer_is_after(self.needcheck) {
            self.check_space = false;

            /* Don't fill the ac if nothing will be written */
            if !self.active {
                return;
            }
        } else {
            self.check_space = true;
        }

        ptr::write_bytes(ac, 0, 1);

        // Populate version
        #[cfg(feature = "acct_v3")]
        { ac.ac_version = ACCT_VERSION_3 | ACCT_BYTEORDER; }
        #[cfg(not(feature = "acct_v3"))]
        { ac.ac_version = ACCT_VERSION_1_2 | ACCT_BYTEORDER; }
    
        ac.ac_comm.copy_from_slice_truncated(cur_task.comm());

        // Calculate timers
        let mut run_time: u64 = ktime_get_ns();
        run_time -= (&cur_task.group_leader).start_time();

        let mut elapsed = nsec_to_ahz(run_time);

        #[cfg(feature = "acct_v3")]
            {
                ac.ac_etime = encode_float(elapsed);
            }
            #[cfg(not(feature = "acct_v3"))]
            {
                ac.ac_etime = encode_comp_t(if elapsed < usize::MAX as u64 { elapsed } else { usize::MAX as u64 });
            }

            #[cfg(any(feature = "acct_v1", feature = "acct_v2"))]
            {
                let etime = encode_comp2_t(elapsed);
                ac.ac_etime_hi = (etime >> 16) as u16;
                ac.ac_etime_lo = etime as u16;
            }

            // C's do_div(elapsed, AHZ) modifies elapsed in-place to equal (elapsed / AHZ)
            let btime = ktime_get_real_seconds() - (elapsed / AHZ);
            ac.ac_btime = btime.clamp(0, u32::MAX as i64) as u32;

            #[cfg(feature = "acct_v2")]
            {
                ac.ac_ahz = AHZ;
            }
    }

    fn write_process(&mut self) {
        // TODO: Implementation
    }
}