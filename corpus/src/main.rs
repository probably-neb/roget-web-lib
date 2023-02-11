use rayon::prelude::*;
use std::collections::HashMap;
use std::collections::binary_heap;
use std::io::prelude::*;
use std::io::BufReader;
use std::num::NonZeroUsize;

const MAX_WORDS: usize = 1500;

// const DICTIONARY: &str = include_str!("../wordle.txt");
#[derive(Eq, PartialEq, Hash, Ord, PartialOrd)]
struct Entry {
    count: usize,
    word: Vec<u8>,
}

fn main() {
    let len: usize = std::env::args()
        .nth(1)
        .expect("no size given")
        .parse()
        .expect("invalid size given");
    eprintln!("Parsing Words of length: {}", len);
    let files: Vec<_> = std::env::args().skip(2).collect();
    let words: HashMap<_, usize> = files
        .into_par_iter()
        .map(|file| {
            let file = std::fs::File::open(&file)
                .unwrap_or_else(|e| panic!("could not open file '{}': {}", file, e));
            let file = BufReader::new(file);
            let file = flate2::bufread::GzDecoder::new(file);
            let mut file = BufReader::new(file);
            let mut words: HashMap<_, _> = HashMap::new(); // DICTIONARY.lines().map(|w| (w.as_bytes(), 0)).collect();
            let mut line = Vec::new();
            loop {
                line.clear();
                if file
                    .read_until(b'\n', &mut line)
                    .expect("reading from stdin should be okay")
                    == 0
                {
                    break;
                }
                let mut fields = line.split_mut(|&c| c == b'\t');
                let word = fields.next().expect("every line should have a word");
                let word = if let Some(w) = word.splitn_mut(2, |&c| c == b'_').next() {
                    w
                } else {
                    word
                };
                if word.len() != len {
                    // eprintln!("cleared line: {:?}", std::str::from_utf8(&word));
                    line.clear();
                    continue;
                }
                if !word.iter().all(|c| matches!(c, b'a'..=b'z' | b'A'..=b'Z')) {
                    continue;
                }
                word.make_ascii_lowercase();
                // if let Some(accum) = words.get_mut(&*word) {
                let count: usize = fields
                    .map(|field| {
                        let mut columns = field.split(|&c| c == b',');
                        let count = columns.nth(1).expect("every row has three fields");
                        let mut v = 0;
                        let mut dec = 1;
                        for &digit in count.iter().rev() {
                            assert!(matches!(digit, b'0'..=b'9'));
                            let digit = digit - b'0';
                            v += digit as usize * dec;
                            dec *= 10;
                        }
                        v
                    })
                    .sum();
                words
                    .entry(word.to_vec())
                    .and_modify(|accum| {
                        *accum += count;
                    })
                    .or_insert(count);
            }
            words
        })
        .reduce(HashMap::new, |mut map1, map2| {
            for (word, count) in map2 {
                *map1.entry(word).or_insert(0) += count;
            }
            map1
        });
    let bheap = words
        .into_iter()
        .map(|(word, count)| Entry { word, count })
        .collect::<binary_heap::BinaryHeap<_>>();

    let words = bheap
        .into_iter()
        .take(MAX_WORDS)
        // .map(|e| e.word)
        // .map(String::from_utf8)
        // .map(Result::unwrap)
        .collect::<Vec<_>>();

    // words.sort();


    // for word in DICTIONARY.lines() {
    //     let count = words
    //         .get(word.as_bytes())
    //         .copied()
    //         .and_then(NonZeroUsize::new)
    //         .map(|v| v.into())
    //         .unwrap_or(1);
    //     println!("{} {}", word, count);
    // }
    for Entry { count , word} in words.iter() {
        let word = std::str::from_utf8(word).expect("word should be ascii");
        println!("{} {}", word, count);
    }
    // for word in words.iter() {
    //     // let word = std::str::from_utf8(word).expect("word should be ascii");
    //     println!("{}", word);
    // }
}
