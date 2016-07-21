extern crate unindent;

use common::util::*;
use std::path::Path;
use std::env;
use std::io::Write;
use std::fs::File;
use std::fs::remove_file;
use self::unindent::*;


// octal dump of 'abcdefghijklmnopqrstuvwxyz\n'
static ALPHA_OUT: &'static str = "
        0000000    061141  062143  063145  064147  065151  066153  067155  070157
        0000020    071161  072163  073165  074167  075171  000012                
        0000033
        ";

// XXX We could do a better job of ensuring that we have a fresh temp dir to ourself,
// not a general one ful of other proc's leftovers.

// Test that od can read one file and dump with default format
#[test]
fn test_file() {
    use std::env;
    let temp = env::temp_dir();
    let tmpdir = Path::new(&temp);
    let file = tmpdir.join("test");

    {
        let mut f = File::create(&file).unwrap();
        match f.write_all(b"abcdefghijklmnopqrstuvwxyz\n") {
            Err(_)  => panic!("Test setup failed - could not write file"),
            _ => {}
        }
    }

    let result = new_ucmd!().arg(file.as_os_str()).run();

    assert_empty_stderr!(result);
    assert!(result.success);
    assert_eq!(result.stdout, unindent(ALPHA_OUT));

    let _ = remove_file(file);
}

// Test that od can read 2 files and concatenate the contents
#[test]
fn test_2files() {
    let temp = env::temp_dir();
    let tmpdir = Path::new(&temp);
    let file1 = tmpdir.join("test1");
    let file2 = tmpdir.join("test2");

    for &(n,a) in [(1,"a"), (2,"b")].iter() {
        println!("number: {} letter:{}", n, a);
     }

    for &(path,data)in &[(&file1, "abcdefghijklmnop"),(&file2, "qrstuvwxyz\n")] {
        let mut f = File::create(&path).unwrap();
        match f.write_all(data.as_bytes()) {
            Err(_)  => panic!("Test setup failed - could not write file"),
            _ => {}
        }
    }

    let result = new_ucmd!().arg(file1.as_os_str()).arg(file2.as_os_str()).run();

    assert_empty_stderr!(result);
    assert!(result.success);
    assert_eq!(result.stdout, unindent(ALPHA_OUT));

    let _ = remove_file(file1);
    let _ = remove_file(file2);
}

// Test that od gives non-0 exit val for filename that dosen't exist.
#[test]
fn test_no_file() {
    let temp = env::temp_dir();
    let tmpdir = Path::new(&temp);
    let file = tmpdir.join("}surely'none'would'thus'a'file'name");

    let result = new_ucmd!().arg(file.as_os_str()).run();

    assert!(!result.success);
}

// Test that od reads from stdin instead of a file
#[test]
fn test_from_stdin() {

    let input = "abcdefghijklmnopqrstuvwxyz\n";
    let result = new_ucmd!().run_piped_stdin(input.as_bytes());

    assert_empty_stderr!(result);
    assert!(result.success);
    assert_eq!(result.stdout, unindent(ALPHA_OUT));

}

// Test that od reads from stdin and also from files
#[test]
fn test_from_mixed() {

    let temp = env::temp_dir();
    let tmpdir = Path::new(&temp);
    let file1 = tmpdir.join("test-1");
    let file3 = tmpdir.join("test-3");

    let (data1, data2, data3) = ("abcdefg","hijklmnop","qrstuvwxyz\n");
    for &(path,data)in &[(&file1, data1),(&file3, data3)] {
        let mut f = File::create(&path).unwrap();
        match f.write_all(data.as_bytes()) {
            Err(_)  => panic!("Test setup failed - could not write file"),
            _ => {}
        }
    }

    let result = new_ucmd!().arg(file1.as_os_str()).arg("--").arg(file3.as_os_str()).run_piped_stdin(data2.as_bytes());

    assert_empty_stderr!(result);
    assert!(result.success);
    assert_eq!(result.stdout, unindent(ALPHA_OUT));

}

#[test]
fn test_multiple_formats() {

    let input = "abcdefghijklmnopqrstuvwxyz\n";
    let result = new_ucmd!().arg("-c").arg("-b").run_piped_stdin(input.as_bytes());

    assert_empty_stderr!(result);
    assert!(result.success);
    assert_eq!(result.stdout, unindent("
            0000000    a   b   c   d   e   f   g   h   i   j   k   l   m   n   o   p
                      141 142 143 144 145 146 147 150 151 152 153 154 155 156 157 160
            0000020    q   r   s   t   u   v   w   x   y   z  \\n                
                      161 162 163 164 165 166 167 170 171 172 012                
            0000033
            "));

}

#[test]
fn test_dec() {

    let input = [
    	0u8, 0u8,
    	1u8, 0u8,
    	2u8, 0u8,
    	3u8, 0u8,
    	0xffu8,0x7fu8,
    	0x00u8,0x80u8,
    	0x01u8,0x80u8,];
    let expected_output = unindent("
            0000000         0       1       2       3   32767  -32768  -32767        
            0000016
            ");
    let result = new_ucmd!().arg("-i").run_piped_stdin(&input[..]);

    assert_empty_stderr!(result);
    assert!(result.success);
    assert_eq!(result.stdout, expected_output);

}

#[test]
fn test_f32(){

    let input : [u8; 24] = [
        0x52, 0x06, 0x9e, 0xbf, // 0xbf9e0652 -1.2345679
        0x4e, 0x61, 0x3c, 0x4b, // 0x4b3c614e 12345678
        0x0f, 0x9b, 0x94, 0xfe, // 0xfe949b0f -9.876543E37
        0x00, 0x00, 0x00, 0x80, // 0x80000000 -0.0
        0xff, 0xff, 0xff, 0x7f, // 0x7fffffff NaN
        0x00, 0x00, 0x7f, 0x80];// 0x807f0000 -1.1663108E-38
    let expected_output = unindent("
            0000000     -1.2345679       12345678  -9.8765427e37             -0
            0000020            NaN -1.1663108e-38                                
            0000030
            ");
    let result = new_ucmd!().arg("-f").run_piped_stdin(&input[..]);

    assert_empty_stderr!(result);
    assert!(result.success);
    assert_eq!(result.stdout, expected_output);
}

// We don't support multibyte chars, so big NEIN to this
/*
#[test]
fn mit_die_umlauten_getesten() {
    let result = new_ucmd!()
        .run_piped_stdin("Universität Tübingen".as_bytes());
    assert_empty_stderr!(result);
    assert!(result.success);
    assert_eq!(result.stdout,
    "0000000    U   n   i   v   e   r   s   i   t   ä  **   t       T   ü  **\n0000020    b   i   n   g   e   n\n0000026")
}
*/
