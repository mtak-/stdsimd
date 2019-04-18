#[cfg(test)]
use stdsimd_test::assert_instr;

extern "C" {
    #[link_name = "llvm.x86.xbegin"]
    fn x86_xbegin() -> i32;
    #[link_name = "llvm.x86.xend"]
    fn x86_xend() -> ();
    #[link_name = "llvm.x86.xabort"]
    fn x86_xabort(imm8: i8) -> ();
    #[link_name = "llvm.x86.xtest"]
    fn x86_xtest() -> i32;
}

/// Transaction successfully started.
pub const _XBEGIN_STARTED: u32 = !0;

/// Transaction explicitly aborted with xabort. The parameter passed to xabort is available with
/// _xabort_code(status).
pub const _XABORT_EXPLICIT: u32 = 1 << 0;

/// Transaction retry is possible.
pub const _XABORT_RETRY: u32 = 1 << 1;

/// Transaction abort due to a memory conflict with another thread.
pub const _XABORT_CONFLICT: u32 = 1 << 2;

/// Transaction abort due to the transaction using too much memory.
pub const _XABORT_CAPACITY: u32 = 1 << 3;

/// Transaction abort due to a debug trap.
pub const _XABORT_DEBUG: u32 = 1 << 4;

/// Transaction abort in a inner nested transaction.
pub const _XABORT_NESTED: u32 = 1 << 5;

/// Specifies the start of a restricted transactional memory (RTM) code region and returns a value
/// indicating status.
///
/// [Intel's documentation](https://software.intel.com/en-us/cpp-compiler-developer-guide-and-reference-xbegin).
#[inline]
#[target_feature(enable = "rtm")]
#[cfg_attr(test, assert_instr(xbegin))]
pub unsafe fn _xbegin() -> u32 {
    x86_xbegin() as _
}

/// Specifies the end of a restricted transactional memory (RTM) code region.
///
/// [Intel's documentation](https://software.intel.com/en-us/cpp-compiler-developer-guide-and-reference-xend).
#[inline]
#[target_feature(enable = "rtm")]
#[cfg_attr(test, assert_instr(xend))]
pub unsafe fn _xend() {
    x86_xend()
}

/// Forces a restricted transactional memory (RTM) region to abort.
///
/// [Intel's documentation](https://software.intel.com/en-us/cpp-compiler-developer-guide-and-reference-xabort).
#[inline]
#[target_feature(enable = "rtm")]
#[cfg_attr(test, assert_instr(xabort))]
pub unsafe fn _xabort(imm8: u32) {
    macro_rules! call {
        ($imm8:expr) => {
            x86_xabort($imm8)
        };
    }
    constify_imm8!(imm8, call)
}

/// Queries whether the processor is executing in a transactional region identified by restricted
/// transactional memory (RTM) or hardware lock elision (HLE).
///
/// [Intel's documentation](https://software.intel.com/en-us/cpp-compiler-developer-guide-and-reference-xtest).
#[inline]
#[target_feature(enable = "rtm")]
#[cfg_attr(test, assert_instr(xtest))]
pub unsafe fn _xtest() -> bool {
    x86_xtest() != 0
}

/// Retrieves the parameter passed to [`_xabort`] when [`_xbegin`]'s status has the `_XABORT_EXPLICIT` flag set.
#[inline]
pub fn _xabort_code(status: u32) -> u32 {
    (status >> 24) & 0xFF
}

#[cfg(test)]
mod tests {
    use crate::core_arch::x86::*;

    #[test]
    fn test_xbegin_xend() {
        unsafe {
            let mut x = 0;
            for _ in 0..10 {
                let code = rtm::_xbegin();
                if code == _XBEGIN_STARTED {
                    x += 1;
                    rtm::_xend();
                    assert_eq!(x, 1);
                    break
                }
                assert_eq!(x, 0);
            }
        }
    }

    #[test]
    fn test_xabort() {
        unsafe {
            // aborting with outside a transactional region does nothing
            _xabort(0);

            for abort_code in 0..10 {
                let mut x = 0;
                let code = rtm::_xbegin();
                if code == _XBEGIN_STARTED {
                    x += 1;
                    rtm::_xabort(abort_code);
                } else if code & _XABORT_EXPLICIT != 0 {
                    let test_abort_code = rtm::_xabort_code(code);
                    assert_eq!(test_abort_code, abort_code);
                }
                assert_eq!(x, 0);
            }
        }
    }

    #[test]
    fn test_xtest() {
        unsafe {
            assert_eq!(_xtest(), false);

            for _ in 0..10 {
                let code = rtm::_xbegin();
                if code == _XBEGIN_STARTED {
                    let in_tx = _xtest();
                    rtm::_xend();
                    
                    // putting the assert inside the transaction would abort the transaction on fail
                    // without any output/panic/etc
                    assert_eq!(in_tx, true);
                    break
                }
            }
        }
    }
}