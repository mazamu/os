#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::mmap;

/*
理想结果：输出 Test 04_1 OK!
*/

#[no_mangle]
fn main() -> i32 {
    let start: usize = 0x10000000;
    let len: usize = 4096;//4KB
    let prot: usize = 3;//11
    //prot：第 0 位表示是否可读，第 1 位表示是否可写，第 2 位表示是否可执行。其他位无效且必须为 0
    assert_eq!(0, mmap(start, len, prot));
    //在已经申请空间的地址里填入内容
    for i in start..(start + len) {
        let addr: *mut u8 = i as *mut u8;
        unsafe {
            *addr = i as u8;
        }
    }

    //从地址里拿出内容比较
    for i in start..(start + len) {
        let addr: *mut u8 = i as *mut u8;
        unsafe {
            assert_eq!(*addr, i as u8);
        }
    }
    println!("Test 04_1 OK!");
    0
}
