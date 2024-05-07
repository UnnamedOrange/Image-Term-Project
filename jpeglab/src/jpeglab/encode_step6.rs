use std::io;

use bitvec::prelude::*;
use lazy_static::lazy_static;

use super::encode_step5::ZigzagDu;
use super::encode_step5::ZigzagMcuCollection;

/// 按 JPEG 标准定义霍夫曼码表结构体，由长度表和符号表组成，描述了一棵霍夫曼树。
/// 编码 DC 的数字时，会根据数字的大小分为至多 16 个符号，这些符号用这里定义的霍夫曼码表编码。见课件表 8.17, 8.18。
/// 编码 AC 的数字时，会根据数字的大小或者行程编码 0 的数量分为很多符号。见课件表 8.17, 8.19。
#[derive(Debug)]
pub struct JpegHuffmanTable {
    /// 长度为 (n + 1) 的霍夫曼码字有 `codes[n]` 个。
    /// 共有 `self.codes.iter().map(|&x| x as usize).sum::<usize>()` 个霍夫曼码字。
    /// 只使用 `codes` 就可以生成出霍夫曼树，因为是范式霍夫曼编码，是按一定规则生成的，见 `generate_bits`。
    pub codes: [u8; 16],
    /// 使用 `codes` 按范式霍夫曼编码依次生成的霍夫曼码字对应的符号，每个符号占一个字节。
    pub values: Vec<u8>,
}

// 完整的亮度直流、亮度交流、色度直流、色度交流的默认霍夫曼码表参见：
// https://blog.csdn.net/xiaoyafang123/article/details/120370880

const LUMINANCE_DC: &str = r#"0	2	00
1	3	010
2	3	011
3	3	100
4	3	101
5	3	110
6	4	1110
7	5	11110
8	6	111110
9	7	1111110
A	8	11111110
B	9	111111110"#;

const CHROMA_DC: &str = r#"0	2	00
1	2	01
2	2	10
3	3	110
4	4	1110
5	5	11110
6	6	111110
7	7	1111110
8	8	11111110
9	9	111111110
A	10	1111111110
B	11	11111111110"#;

const LUMINANCE_AC: &str = r#"00	4	1010
01	2	00
02	2	01
03	3	100
04	4	1011
05	5	11010
06	7	1111000
07	8	11111000
08	10	1111110110
09	16	1111111110000010
0A	16	1111111110000011
11	4	1100
12	5	11011
13	7	1111001
14	9	111110110
15	11	11111110110
16	16	1111111110000100
17	16	1111111110000101
18	16	1111111110000110
19	16	1111111110000111
1A	16	1111111110001000
21	5	11100
22	8	11111001
23	10	1111110111
24	12	111111110100
25	16	1111111110001001
26	16	1111111110001010
27	16	1111111110001011
28	16	1111111110001100
29	16	1111111110001101
2A	16	1111111110001110
31	6	111010
32	9	111110111
33	12	111111110101
34	16	1111111110001111
35	16	1111111110010000
36	16	1111111110010001
37	16	1111111110010010
38	16	1111111110010011
39	16	1111111110010100
3A	16	1111111110010101
41	6	111011
42	10	1111111000
43	16	1111111110010110
44	16	1111111110010111
45	16	1111111110011000
46	16	1111111110011001
47	16	1111111110011010
48	16	1111111110011011
49	16	1111111110011100
4A	16	1111111110011101
51	7	1111010
52	11	11111110111
53	16	1111111110011110
54	16	1111111110011111
55	16	1111111110100000
56	16	1111111110100001
57	16	1111111110100010
58	16	1111111110100011
59	16	1111111110100100
5A	16	1111111110100101
61	7	1111011
62	12	111111110110
63	16	1111111110100110
64	16	1111111110100111
65	16	1111111110101000
66	16	1111111110101001
67	16	1111111110101010
68	16	1111111110101011
69	16	1111111110101100
6A	16	1111111110101101
71	8	11111010
72	12	111111110111
73	16	1111111110101110
74	16	1111111110101111
75	16	1111111110110000
76	16	1111111110110001
77	16	1111111110110010
78	16	1111111110110011
79	16	1111111110110100
7A	16	1111111110110101
81	9	111111000
82	15	111111111000000
83	16	1111111110110110
84	16	1111111110110111
85	16	1111111110111000
86	16	1111111110111001
87	16	1111111110111010
88	16	1111111110111011
89	16	1111111110111100
8A	16	1111111110111101
91	9	111111001
92	16	1111111110111110
93	16	1111111110111111
94	16	1111111111000000
95	16	1111111111000001
96	16	1111111111000010
97	16	1111111111000011
98	16	1111111111000100
99	16	1111111111000101
9A	16	1111111111000110
A1	9	111111010
A2	16	1111111111000111
A3	16	1111111111001000
A4	16	1111111111001001
A5	16	1111111111001010
A6	16	1111111111001011
A7	16	1111111111001100
A8	16	1111111111001101
A9	16	1111111111001110
AA	16	1111111111001111
B1	10	1111111001
B2	16	1111111111010000
B3	16	1111111111010001
B4	16	1111111111010010
B5	16	1111111111010011
B6	16	1111111111010100
B7	16	1111111111010101
B8	16	1111111111010110
B9	16	1111111111010111
BA	16	1111111111011000
C1	10	1111111010
C2	16	1111111111011001
C3	16	1111111111011010
C4	16	1111111111011011
C5	16	1111111111011100
C6	16	1111111111011101
C7	16	1111111111011110
C8	16	1111111111011111
C9	16	1111111111100000
CA	16	1111111111100001
D1	11	11111111000
D2	16	1111111111100010
D3	16	1111111111100011
D4	16	1111111111100100
D5	16	1111111111100101
D6	16	1111111111100110
D7	16	1111111111100111
D8	16	1111111111101000
D9	16	1111111111101001
DA	16	1111111111101010
E1	16	1111111111101011
E2	16	1111111111101100
E3	16	1111111111101101
E4	16	1111111111101110
E5	16	1111111111101111
E6	16	1111111111110000
E7	16	1111111111110001
E8	16	1111111111110010
E9	16	1111111111110011
EA	16	1111111111110100
F0	11	11111111001
F1	16	1111111111110101
F2	16	1111111111110110
F3	16	1111111111110111
F4	16	1111111111111000
F5	16	1111111111111001
F6	16	1111111111111010
F7	16	1111111111111011
F8	16	1111111111111100
F9	16	1111111111111101
FA	16	1111111111111110"#;

const CHROMA_AC: &str = r#"00	2	00
01	2	01
02	3	100
03	4	1010
04	5	11000
05	5	11001
06	6	111000
07	7	1111000
08	9	111110100
09	10	1111110110
0A	12	111111110100
11	4	1011
12	6	111001
13	8	11110110
14	9	111110101
15	11	11111110110
16	12	111111110101
17	16	1111111110001000
18	16	1111111110001001
19	16	1111111110001010
1A	16	1111111110001011
21	5	11010
22	8	11110111
23	10	1111110111
24	12	111111110110
25	15	111111111000010
26	16	1111111110001100
27	16	1111111110001101
28	16	1111111110001110
29	16	1111111110001111
2A	16	1111111110010000
31	5	11011
32	8	11111000
33	10	1111111000
34	12	111111110111
35	16	1111111110010001
36	16	1111111110010010
37	16	1111111110010011
38	16	1111111110010100
39	16	1111111110010101
3A	16	1111111110010110
41	6	111010
42	9	111110110
43	16	1111111110010111
44	16	1111111110011000
45	16	1111111110011001
46	16	1111111110011010
47	16	1111111110011011
48	16	1111111110011100
49	16	1111111110011101
4A	16	1111111110011110
51	6	111011
52	10	1111111001
53	16	1111111110011111
54	16	1111111110100000
55	16	1111111110100001
56	16	1111111110100010
57	16	1111111110100011
58	16	1111111110100100
59	16	1111111110100101
5A	16	1111111110100110
61	7	1111001
62	11	11111110111
63	16	1111111110100111
64	16	1111111110101000
65	16	1111111110101001
66	16	1111111110101010
67	16	1111111110101011
68	16	1111111110101100
69	16	1111111110101101
6A	16	1111111110101110
71	7	1111010
72	11	11111111000
73	16	1111111110101111
74	16	1111111110110000
75	16	1111111110110001
76	16	1111111110110010
77	16	1111111110110011
78	16	1111111110110100
79	16	1111111110110101
7A	16	1111111110110110
81	8	11111001
82	16	1111111110110111
83	16	1111111110111000
84	16	1111111110111001
85	16	1111111110111010
86	16	1111111110111011
87	16	1111111110111100
88	16	1111111110111101
89	16	1111111110111110
8A	16	1111111110111111
91	9	111110111
92	16	1111111111000000
93	16	1111111111000001
94	16	1111111111000010
95	16	1111111111000011
96	16	1111111111000100
97	16	1111111111000101
98	16	1111111111000110
99	16	1111111111000111
9A	16	1111111111001000
A1	9	111111000
A2	16	1111111111001001
A3	16	1111111111001010
A4	16	1111111111001011
A5	16	1111111111001100
A6	16	1111111111001101
A7	16	1111111111001110
A8	16	1111111111001111
A9	16	1111111111010000
AA	16	1111111111010001
B1	9	111111001
B2	16	1111111111010010
B3	16	1111111111010011
B4	16	1111111111010100
B5	16	1111111111010101
B6	16	1111111111010110
B7	16	1111111111010111
B8	16	1111111111011000
B9	16	1111111111011001
BA	16	1111111111011010
C1	9	111111010
C2	16	1111111111011011
C3	16	1111111111011100
C4	16	1111111111011101
C5	16	1111111111011110
C6	16	1111111111011111
C7	16	1111111111100000
C8	16	1111111111100001
C9	16	1111111111100010
CA	16	1111111111100011
D1	11	11111111001
D2	16	1111111111100100
D3	16	1111111111100101
D4	16	1111111111100110
D5	16	1111111111100111
D6	16	1111111111101000
D7	16	1111111111101001
D8	16	1111111111101010
D9	16	1111111111101011
DA	16	1111111111101100
E1	14	11111111100000
E2	16	1111111111101101
E3	16	1111111111101110
E4	16	1111111111101111
E5	16	1111111111110000
E6	16	1111111111110001
E7	16	1111111111110010
E8	16	1111111111110011
E9	16	1111111111110100
EA	16	1111111111110101
F0	10	1111111010
F1	15	111111111000011
F2	16	1111111111110110
F3	16	1111111111110111
F4	16	1111111111111000
F5	16	1111111111111001
F6	16	1111111111111010
F7	16	1111111111111011
F8	16	1111111111111100
F9	16	1111111111111101
FA	16	1111111111111110"#;

/// 根据参考网址的默认霍夫曼码表生成我的霍夫曼码表结构体。
/// 返回的元组的第二个是用于验证的码字，按码表的 values 排序。
fn generate_huffman_table(content: &str) -> (JpegHuffmanTable, Vec<BitVec>) {
    let mut codes = [0_u8; 16];
    let mut values = vec![];
    let mut bits_vec = vec![];
    let mut line_data = vec![];
    for line in content.split('\n') {
        let mut symbol = u8::default();
        let mut length = u8::default();
        let mut bits = bitvec![];
        let mut idx = 0;
        for element in line.split('\t') {
            match idx {
                0 => {
                    symbol = u8::from_str_radix(element, 16).unwrap();
                }
                1 => {
                    length = element.parse::<u8>().unwrap();
                }
                2 => {
                    for ch in element.chars() {
                        match ch {
                            '0' => {
                                bits.push(false);
                            }
                            '1' => {
                                bits.push(true);
                            }
                            _ => {
                                panic!();
                            }
                        }
                    }
                }
                _ => {
                    break;
                }
            }
            idx += 1;
        }
        codes[(length - 1) as usize] += 1;
        line_data.push((symbol, length, bits));
    }

    for length in 0..codes.len() {
        let length = (length + 1) as u8;
        for line in &line_data {
            if line.1 == length {
                values.push(line.0);
                bits_vec.push(line.2.clone());
            }
        }
    }

    (JpegHuffmanTable { codes, values }, bits_vec)
}

impl JpegHuffmanTable {
    /// 根据霍夫曼码表的 codes 字段生成霍夫曼码，values[i] 的霍夫曼码为 ret[i]。
    pub fn generate_bits(&self) -> Vec<BitVec> {
        let mut ret = vec![];
        let mut c = 0_usize;
        for i in 0..self.codes.len() {
            let length = i + 1;
            for _ in 0..self.codes[i] {
                let mut bits = c.view_bits::<Lsb0>().to_owned();
                bits.resize(length, false);
                bits.reverse();
                ret.push(bits);
                c += 1;
            }
            c *= 2;
        }
        ret
    }
}

lazy_static! {
    pub static ref DEFAULT_LUMINANCE_DC_HUFFMAN_TABLE: JpegHuffmanTable =
        generate_huffman_table(LUMINANCE_DC).0;
    pub static ref DEFAULT_CHROMA_DC_HUFFMAN_TABLE: JpegHuffmanTable =
        generate_huffman_table(CHROMA_DC).0;
    pub static ref DEFAULT_LUMINANCE_AC_HUFFMAN_TABLE: JpegHuffmanTable =
        generate_huffman_table(LUMINANCE_AC).0;
    pub static ref DEFAULT_CHROMA_AC_HUFFMAN_TABLE: JpegHuffmanTable =
        generate_huffman_table(CHROMA_AC).0;
}

/// DC 编码器的差分性质由相邻 MCU 之间的同种类 DU 使用，YUV422 共需要 3 个 DC 编码器状态。
struct DcEncoder<'a> {
    pub pred: i16,
    pub huffman_table: &'a JpegHuffmanTable,
}

/// AC 编码器的行程编码性质在单个 DU 内部使用，有多少个 DU 就需要新建多少个 AC 编码器状态。
struct AcEncoder<'a> {
    pub zero_run_length: usize,
    pub huffman_table: &'a JpegHuffmanTable,
}

impl<'a> DcEncoder<'a> {
    pub fn new(huffman_table: &'a JpegHuffmanTable) -> Self {
        Self {
            pred: 0,
            huffman_table,
        }
    }
}

impl<'a> AcEncoder<'a> {
    pub fn new(huffman_table: &'a JpegHuffmanTable) -> Self {
        Self {
            zero_run_length: 0,
            huffman_table,
        }
    }
}

trait JpegScanEncode {
    fn next(&mut self, value: i16) -> BitVec;
}

impl<'a> JpegScanEncode for DcEncoder<'a> {
    fn next(&mut self, value: i16) -> BitVec {
        todo!()
    }
}

impl<'a> JpegScanEncode for AcEncoder<'a> {
    fn next(&mut self, value: i16) -> BitVec {
        todo!()
    }
}

/// 最基本的 JPEG 编码结果，可以据此生成 JPEG 文件。
/// 但是注意，假设使用 YUV422 采样，使用了默认量化表，使用了默认霍夫曼码表，这些都不在此提及。
pub struct JpegOutputData {
    pub original_width: usize,
    pub original_height: usize,
    /// 熵编码的最终结果。
    pub scan: BitVec,
}

/// 第六步：编码。
/// 分为直流和交流。
/// 为了方便，熵编码使用默认的霍夫曼编码。
/// 尽管 DC 分量有差分编码，仍然是以 DU 为单位进行编码的。
pub fn encode_step6(zigzag_mcu_collection: &ZigzagMcuCollection) -> io::Result<JpegOutputData> {
    let mut scan = bitvec![];
    let mcus = &zigzag_mcu_collection.zigzag_mcus;

    fn encode_du(
        du: &ZigzagDu,
        dc_encoder: &mut DcEncoder,
        ac_huffman_table: &JpegHuffmanTable,
    ) -> BitVec {
        let mut ret = bitvec![];

        let mut ac_encoder = AcEncoder::new(&ac_huffman_table);
        ret.append(&mut dc_encoder.next(du.0[0]));
        for i in 1..du.0.len() {
            ret.append(&mut ac_encoder.next(du.0[i]));
        }

        ret
    }

    let mut dc_encoder_y = DcEncoder::new(&DEFAULT_LUMINANCE_DC_HUFFMAN_TABLE);
    let mut dc_encoder_u = DcEncoder::new(&DEFAULT_CHROMA_DC_HUFFMAN_TABLE);
    let mut dc_encoder_v = DcEncoder::new(&DEFAULT_CHROMA_DC_HUFFMAN_TABLE);
    for mcu in mcus {
        scan.append(&mut encode_du(
            &mcu.y0,
            &mut dc_encoder_y,
            &DEFAULT_LUMINANCE_AC_HUFFMAN_TABLE,
        ));
        scan.append(&mut encode_du(
            &mcu.y1,
            &mut dc_encoder_y,
            &DEFAULT_LUMINANCE_AC_HUFFMAN_TABLE,
        ));
        scan.append(&mut encode_du(
            &mcu.cb,
            &mut dc_encoder_u,
            &DEFAULT_CHROMA_AC_HUFFMAN_TABLE,
        ));
        scan.append(&mut encode_du(
            &mcu.cr,
            &mut dc_encoder_v,
            &DEFAULT_CHROMA_AC_HUFFMAN_TABLE,
        ));
    }

    Ok(JpegOutputData {
        original_width: zigzag_mcu_collection.original_width,
        original_height: zigzag_mcu_collection.original_height,
        scan,
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn print_huffman_tables() {
        fn print_huffman_table(table: &JpegHuffmanTable) {
            println!("{:?}", table);
        }
        print_huffman_table(&DEFAULT_LUMINANCE_DC_HUFFMAN_TABLE);
        print_huffman_table(&DEFAULT_CHROMA_DC_HUFFMAN_TABLE);
        print_huffman_table(&DEFAULT_LUMINANCE_AC_HUFFMAN_TABLE);
        print_huffman_table(&DEFAULT_CHROMA_AC_HUFFMAN_TABLE);
    }

    #[test]
    fn test_generate_bits() {
        let (table, bits) = generate_huffman_table(LUMINANCE_DC);
        assert_eq!(table.generate_bits(), bits);

        let (table, bits) = generate_huffman_table(CHROMA_DC);
        assert_eq!(table.generate_bits(), bits);

        let (table, bits) = generate_huffman_table(LUMINANCE_AC);
        assert_eq!(table.generate_bits(), bits);

        let (table, bits) = generate_huffman_table(CHROMA_AC);
        assert_eq!(table.generate_bits(), bits);
    }
}