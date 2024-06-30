// Geohash length	Cell width	Cell height
// 1	≤ 5,000km	×	5,000km
// 2	≤ 1,250km	×	625km
// 3	≤ 156km	×	156km
// 4	≤ 39.1km	×	19.5km
// 5	≤ 4.89km	×	4.89km
// 6	≤ 1.22km	×	0.61km
// 7	≤ 153m	×	153m
// 8	≤ 38.2m	×	19.1m
// 9	≤ 4.77m	×	4.77m
// 10	≤ 1.19m	×	0.596m
// 11	≤ 149mm	×	149mm
// 12	≤ 37.2mm	×	18.6mm

use bitvec::prelude::*;
// use std::error::Error;

const BASE32_CODES: [char; 32] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'j', 'k',
    'm', 'n', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];
const LONGITUDE_BOUND: [f32; 2] = [-90.0, 90.0];
const LATITUDE_BOUND: [f32; 2] = [-180.0, 180.0];
const BIT_VEC_CAPACITY: usize = 32;

#[derive(Debug)]
struct GeoHashError {
    msg: String,
}

fn check_precision(precision: u8) -> bool {
    if precision > 12 {
        return false;
    }
    return true;
}

fn get_bit_representation(position: usize) -> Vec<u8> {
    let mut bits = Vec::with_capacity(5);
    for i in (0..5).rev() {
        bits.push(((position >> i) & 1) as u8);
    }
    bits
}

fn validate_latitude_longitude(latitude: f32, longitude: f32) -> bool {
    if latitude < -90.0 || latitude > 90.0 {
        return false;
    }
    if longitude < -180.0 || longitude > 180.0 {
        return false;
    }
    return true;
}
fn encode(latitude: f32, longitude: f32, precision: u8) -> Result<String, GeoHashError> {
    if !check_precision(precision) {
        return Err(GeoHashError {
            msg: String::from("Precision must be between 1 and 12"),
        });
    }
    if !validate_latitude_longitude(latitude, longitude) {
        return Err(GeoHashError {
            msg: String::from(
                "Latitude must be between -90 and 90, Longitude must be between -180 and 180",
            ),
        });
    }
    // Set capacity of the vector
    let mut lng_bit: Vec<bool> = Vec::with_capacity(BIT_VEC_CAPACITY);
    let mut lat_bit: Vec<bool> = Vec::with_capacity(BIT_VEC_CAPACITY);

    let mut lat_bound = vec![LATITUDE_BOUND[0], 0.0, LATITUDE_BOUND[1]];
    let mut lng_bound = vec![LONGITUDE_BOUND[0], 0.0, LONGITUDE_BOUND[1]];

    for _ in 0..BIT_VEC_CAPACITY {
        //Check if x value is greater than the mid point
        if latitude >= lat_bound[1] {
            lng_bit.push(true);
            lat_bound[0] = lat_bound[1];
            lat_bound[1] = (lat_bound[1] + lat_bound[2]) / 2.0;
        } else {
            lng_bit.push(false);
            lat_bound[2] = lat_bound[1];
            lat_bound[1] = (lat_bound[0] + lat_bound[1]) / 2.0;
        }

        //Check if y value is greater than the mid point
        if longitude >= lng_bound[1] {
            lat_bit.push(true);
            lng_bound[0] = lng_bound[1];
            lng_bound[1] = (lng_bound[1] + lng_bound[2]) / 2.0;
        } else {
            lat_bit.push(false);
            lng_bound[2] = lng_bound[1];
            lng_bound[1] = (lng_bound[0] + lng_bound[1]) / 2.0;
        }
    }

    let lng_bit_vec: BitVec = lng_bit.into_iter().collect();
    let lat_bit_vec: BitVec = lat_bit.into_iter().collect();

    let mut z_dimension_vec: BitVec<u64, Msb0> = BitVec::with_capacity(64);

    for i in 0..BIT_VEC_CAPACITY {
        z_dimension_vec.push(lng_bit_vec[i]);
        z_dimension_vec.push(lat_bit_vec[i]);
    }

    let mut geohash = String::new();
    for chnk in z_dimension_vec.chunks_exact(5) {
        let pos = chnk.load::<u8>() as usize;
        // println!("Encoding {} with pos {}", chnk, pos);
        geohash.push_str(&BASE32_CODES[pos].to_string().as_str());
        if geohash.len() == precision as usize {
            break;
        }
    }
    // println!("Geohash: {:?}", z_dimension_vec);
    return Ok(geohash.to_string());
}

fn decode(geocode: &str) {
    println!("Decoding geocode: {}", geocode);
    let mut bit_vec: Vec<bool> = Vec::with_capacity(BIT_VEC_CAPACITY);

    for (_, ch) in geocode.chars().enumerate() {
        // Even position is latitude, odd is longitude
        println!("Currently decoding {}", ch);
        match BASE32_CODES.iter().position(|&x| x == ch) {
            Some(position) => {
                let bits: Vec<u8> = get_bit_representation(position);
                println!("With position {} and bits {:?}", position, bits);
                bit_vec.extend(bits.iter().map(|&b| b == 1));
            }
            None => {
                GeoHashError {
                    msg: format!("The geohash contain invalid character: {}", ch),
                };
                return;
            }
        }

        // look up position of the character in the BASE32_CODES array
    }
    println!("{:#?}", bit_vec);
}
// fn decode(geocode: &str) {
//     // Determine if the initial position is longitude based on the length of the geocode
//     let mut pos_is_lng = geocode.len() % 2 == 0;
//     println!("Decoding geocode: {}", geocode);

//     // Vectors to store binary representations of longitude and latitude
//     let mut lng_bit: Vec<bool> = Vec::with_capacity(BIT_VEC_CAPACITY);
//     let mut lat_bit: Vec<bool> = Vec::with_capacity(BIT_VEC_CAPACITY);

//     for ch in geocode.chars() {
//         // Find the position of the character in the BASE32_CODES array
//         if let Some(pos) = BASE32_CODES.iter().position(|&x| x == ch) {
//             // Get 5 bit representation of the position
//             let bit = get_bit_representation(pos);
//             // Push the bits to the respective vector
//             for i in 0..5 {
//                 if pos_is_lng {
//                     lng_bit.push(bit[i] == 1);
//                 } else {
//                     lat_bit.push(bit[i] == 1);
//                 }
//             }
//             // Toggle the flag for next character
//             pos_is_lng = !pos_is_lng;
//         } else {
//             // Handle the case where character is not found in BASE32_CODES
//             println!("Invalid character '{}' in geocode.", ch);
//             return;
//         }
//     }

//     // Convert the vectors of bools to BitVec
//     let lng_bit_vec: BitVec = lng_bit.into_iter().collect();
//     let lat_bit_vec: BitVec = lat_bit.into_iter().collect();

//     println!("Longitude Bit Vector: {:?}", lng_bit_vec);
//     println!("Latitude Bit Vector: {:?}", lat_bit_vec);

//     for index in lng_bit_vec.chunks_exact(5) {
//         println!("{:?}", index);
//     }

//     // if lat 0 then
// }

fn main() {
    let latitude: f32 = -0.08635;
    let longitude: f32 = 51.52562;
    let precision: u8 = 5;
    let x = encode(latitude, longitude, precision);
    let geocode = x.unwrap();
    // println!("{:#?}", geocode.clone());
    println!("{:#?}", decode(&geocode));
}

// even is longitude, odd is latitude
