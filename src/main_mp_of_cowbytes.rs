
pub mod size;
pub mod cow_bytes;
pub mod storage_preference;

use crate::cow_bytes::CowBytes;
use crate::cow_bytes::SlicedCowBytes;
use crate::storage_preference::StoragePreference;
use crate::size::StaticSize;

use std::{collections::BTreeMap, mem};

use rkyv::{Archive, Deserialize, Serialize};
use rkyv::ser::{Serializer, serializers::AllocSerializer};
use serde::de::value;

/// Additional information for a single entry. Concerns meta information like
/// the desired storage level of a key.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub struct KeyInfo {
    storage_preference: StoragePreference,
}

impl From<&ArchivedKeyInfo> for KeyInfo {
    fn from(x: &ArchivedKeyInfo) -> Self {
        KeyInfo {
            storage_preference: (&x.storage_preference).into()
        }
    }
}

impl StaticSize for KeyInfo {
    fn static_size() -> usize {
        mem::size_of::<StoragePreference>()
    }
}

impl KeyInfo {
    pub(crate) fn merge_with_upper(self, upper: KeyInfo) -> KeyInfo {
        KeyInfo {
            storage_preference: StoragePreference::choose_faster(
                self.storage_preference,
                upper.storage_preference,
            ),
        }
    }

    pub(crate) fn storage_preference(&self) -> &StoragePreference {
        &self.storage_preference
    }
}

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
#[archive(
    // This will generate a PartialEq impl between our unarchived and archived
    // types:
    //compare(PartialEq),
    // bytecheck can be used to validate your data if you want. To use the safe
    // API, you have to derive CheckBytes for the archived type:
    check_bytes,
)]
// Derives can be passed through to the generated type:
//#[archive_attr(derive(Debug))]
pub struct Type_A {
    #[with(rkyv::with::AsVec)]
    pub entries: BTreeMap<CowBytes, (KeyInfo, SlicedCowBytes)>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Type_B {
    //#[with(rkyv::with::AsVec)]
    pub entries: BTreeMap<CowBytes, (KeyInfo, SlicedCowBytes)>,
}

//const TEXT_FOR_VALUE: &str = "Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum. It is a long established fact that a reader will be distracted by the readable content of a page when looking at its layout. The point of using Lorem Ipsum is that it has a more-or-less normal distribution of letters, as opposed to using 'Content here, content here', making it look like readable English. Many desktop publishing packages and web page editors now use Lorem Ipsum as their default model text, and a search for 'lorem ipsum' will uncover many web sites still in their infancy. Various versions have evolved over the years, sometimes by accident, sometimes on purpose (injected humour and the like).";
//const TEXT_FOR_VALUE: &str = "hello world!";

const ENTRIES_COUNT: u32 = 10000;

fn fill_data_a(data: &mut Type_A) {   
    let TEXT_FOR_VALUE : Vec<u8> =  vec![1u8; 128*1024];
    let value = CowBytes::from(TEXT_FOR_VALUE);
    for number in (1..ENTRIES_COUNT) {
        data.entries.insert(CowBytes::from(number.to_string().as_bytes()),
            (KeyInfo { storage_preference: StoragePreference::new(1) },  SlicedCowBytes::from(value.clone())));
    }
}

fn fill_data_b(data: &mut Type_B) {   
    let TEXT_FOR_VALUE : Vec<u8> =  vec![1u8; 128*1024];
    let value = CowBytes::from(TEXT_FOR_VALUE);
    for number in (1..ENTRIES_COUNT) {
        data.entries.insert(CowBytes::from(number.to_string().as_bytes()),
            (KeyInfo { storage_preference: StoragePreference::new(1) },  SlicedCowBytes::from(value.clone())));
    }
}


fn rkyv_case_a_safe() -> u128 {
    let mut data = Type_A {
        entries: BTreeMap::new()
    };

     let mut cloned_data = Type_B {
       entries: BTreeMap::new()
     };

    fill_data_a(&mut data);
    //let value = CowBytes::from(TEXT_FOR_VALUE.as_bytes());    
    // for number in (1..ENTRIES_COUNT) {
    //     data.entries.insert(CowBytes::from(number.to_string().as_bytes()),
    //         (KeyInfo { storage_preference: StoragePreference::new(1) },  SlicedCowBytes::from(value.clone())));
    // }

    let start_time = std::time::Instant::now();

    let mut serializer = AllocSerializer::<0>::default();
    serializer.serialize_value(&data).unwrap();
    let bytes = serializer.into_serializer().into_inner();

    let archived = rkyv::check_archived_root::<Type_A>(&bytes[..]).unwrap();

    let deserialized: Type_A = archived.deserialize(&mut rkyv::de::deserializers::SharedDeserializeMap::new()).unwrap();

    // for (key, value) in deserialized.entries.iter() {
    //     cloned_data.entries.insert(key.clone(), value.clone());
    // }

    // let mut total_entries_fetched = 1;
    // for (key, value) in deserialized.entries.iter() {
    //     total_entries_fetched = total_entries_fetched + 1;
    //     //println!("{:?}", String::from_utf8(key.clone().into_vec()).unwrap());
    // }
    // assert_eq!(total_entries_fetched, ENTRIES_COUNT);

    start_time.elapsed().as_millis()
}

fn rkyv_case_a_unsafe() -> u128 {
    let mut data = Type_A {
        entries: BTreeMap::new()
    };

    let mut cloned_data = Type_B {
        entries: BTreeMap::new()
    };

    fill_data_a(&mut data);
    // let value = CowBytes::from(TEXT_FOR_VALUE.as_bytes());
    
    // for number in (1..ENTRIES_COUNT) {
    //     data.entries.insert(CowBytes::from(number.to_string().as_bytes()),
    //         (KeyInfo { storage_preference: StoragePreference::new(1) },  SlicedCowBytes::from(value.clone())));
    // }

    let start_time = std::time::Instant::now();

    let mut serializer = AllocSerializer::<0>::default();
    serializer.serialize_value(&data).unwrap();
    let bytes = serializer.into_serializer().into_inner();

    let archived = unsafe { rkyv::archived_root::<Type_A>(&bytes[..]) };

    let deserialized: Type_A = archived.deserialize(&mut rkyv::de::deserializers::SharedDeserializeMap::new()).unwrap();

    // for (key, value) in deserialized.entries.iter() {
    //     cloned_data.entries.insert(key.clone(), value.clone());
    // }

    // let mut total_entries_fetched = 1;
    // for (key, value) in deserialized.entries.iter() {
    //     total_entries_fetched = total_entries_fetched + 1;
    //     //println!("{:?}", String::from_utf8(key.clone().into_vec()).unwrap());
    // }
    // assert_eq!(total_entries_fetched, ENTRIES_COUNT);

    start_time.elapsed().as_millis()
}

fn rkyv_case_b_safe() -> u128 {
    let mut data = Type_A {
        entries: BTreeMap::new()
    };

    let mut cloned_data = Type_B {
        entries: BTreeMap::new()
    };

    fill_data_a(&mut data);
    // let value = CowBytes::from(TEXT_FOR_VALUE.as_bytes());
    
    // for number in (1..ENTRIES_COUNT) {
    //     data.entries.insert(CowBytes::from(number.to_string().as_bytes()),
    //         (KeyInfo { storage_preference: StoragePreference::new(1) },  SlicedCowBytes::from(value.clone())));
    // }

    let start_time = std::time::Instant::now();

    let mut serializer = AllocSerializer::<0>::default();
    serializer.serialize_value(&data).unwrap();
    let bytes = serializer.into_serializer().into_inner();

    let archived = rkyv::check_archived_root::<Type_A>(&bytes[..]).unwrap();

    for entry in archived.entries.iter() {
        cloned_data.entries.insert(CowBytes::from(&entry.key) /*CowBytes::from(entry.key.inner.to_vec())*/,
        ( (&entry.value.0).into(),  SlicedCowBytes::from(CowBytes::from(entry.value.1.data.inner.to_vec()))));
        //break;
            //entry.value.deserialize(&mut rkyv::de::deserializers::SharedDeserializeMap::new()).unwrap());

        //cloned_data.entries.insert(entry.key.deserialize(&mut rkyv::de::deserializers::SharedDeserializeMap::new()).unwrap(),
        //    entry.value.deserialize(&mut rkyv::de::deserializers::SharedDeserializeMap::new()).unwrap());
    }

    // let mut total_entries_fetched = 1;
    // for (key, value) in cloned_data.entries.iter() {
    //     total_entries_fetched = total_entries_fetched + 1;
    //     //println!("{:?} {:?}", value.0, String::from_utf8(key.clone().into_vec()).unwrap());
    // }
    // assert_eq!(total_entries_fetched, ENTRIES_COUNT);

    start_time.elapsed().as_millis()
}

fn rkyv_case_b_unsafe() -> u128 {
    let mut data = Type_A {
        entries: BTreeMap::new()
    };

    let mut cloned_data = Type_B {
        entries: BTreeMap::new()
    };

    fill_data_a(&mut data);
    // let value = CowBytes::from(TEXT_FOR_VALUE.as_bytes());
    
    // for number in (1..ENTRIES_COUNT) {
    //     data.entries.insert(CowBytes::from(number.to_string().as_bytes()),
    //         (KeyInfo { storage_preference: StoragePreference::new(1) },  SlicedCowBytes::from(value.clone())));
    // }

    let start_time = std::time::Instant::now();

    let mut serializer = AllocSerializer::<0>::default();
    serializer.serialize_value(&data).unwrap();
    let bytes = serializer.into_serializer().into_inner();

    let archived = unsafe { rkyv::archived_root::<Type_A>(&bytes[..]) };

    for entry in archived.entries.iter() {
        cloned_data.entries.insert(CowBytes::from(&entry.key) /*CowBytes::from(entry.key.inner.to_vec())*/,
        ( (&entry.value.0).into(),  SlicedCowBytes::from(CowBytes::from(entry.value.1.data.inner.to_vec()))));
        //break;
            //entry.value.deserialize(&mut rkyv::de::deserializers::SharedDeserializeMap::new()).unwrap());

        //cloned_data.entries.insert(entry.key.deserialize(&mut rkyv::de::deserializers::SharedDeserializeMap::new()).unwrap(),
        //    entry.value.deserialize(&mut rkyv::de::deserializers::SharedDeserializeMap::new()).unwrap());
    }

    // let mut total_entries_fetched = 1;
    // for (key, value) in cloned_data.entries.iter() {
    //     total_entries_fetched = total_entries_fetched + 1;
    //     //println!("{:?}", String::from_utf8(value.1.clone().to_vec()).unwrap());
    // }
    // assert_eq!(total_entries_fetched, ENTRIES_COUNT);

    start_time.elapsed().as_millis()
}

fn rkyv_case_b_safe_access_one_entity() -> u128 {
    let mut data = Type_A {
        entries: BTreeMap::new()
    };

    let mut cloned_data = Type_B {
        entries: BTreeMap::new()
    };

    fill_data_a(&mut data);
    // let value = CowBytes::from(TEXT_FOR_VALUE.as_bytes());
    
    // for number in (1..ENTRIES_COUNT) {
    //     data.entries.insert(CowBytes::from(number.to_string().as_bytes()),
    //         (KeyInfo { storage_preference: StoragePreference::new(1) },  SlicedCowBytes::from(value.clone())));
    // }

    let start_time = std::time::Instant::now();

    let mut serializer = AllocSerializer::<0>::default();
    serializer.serialize_value(&data).unwrap();
    let bytes = serializer.into_serializer().into_inner();

    let archived = rkyv::check_archived_root::<Type_A>(&bytes[..]).unwrap();

    for entry in archived.entries.iter() {
        cloned_data.entries.insert(CowBytes::from(&entry.key) /*CowBytes::from(entry.key.inner.to_vec())*/,
        ( (&entry.value.0).into(),  SlicedCowBytes::from(CowBytes::from(entry.value.1.data.inner.to_vec()))));
        break;
        //break;
            //entry.value.deserialize(&mut rkyv::de::deserializers::SharedDeserializeMap::new()).unwrap());

        //cloned_data.entries.insert(entry.key.deserialize(&mut rkyv::de::deserializers::SharedDeserializeMap::new()).unwrap(),
        //    entry.value.deserialize(&mut rkyv::de::deserializers::SharedDeserializeMap::new()).unwrap());
    }

    // let mut total_entries_fetched = 1;
    // for (key, value) in cloned_data.entries.iter() {
    //     total_entries_fetched = total_entries_fetched + 1;
    //     //println!("{:?} {:?}", value.0, String::from_utf8(key.clone().into_vec()).unwrap());
    // }
    // assert_eq!(total_entries_fetched, ENTRIES_COUNT);

    start_time.elapsed().as_millis()
}

fn rkyv_case_b_unsafe_access_one_entity() -> u128 {
    let mut data = Type_A {
        entries: BTreeMap::new()
    };

    let mut cloned_data = Type_B {
        entries: BTreeMap::new()
    };

    fill_data_a(&mut data);
    // let value = CowBytes::from(TEXT_FOR_VALUE.as_bytes());
    
    // for number in (1..ENTRIES_COUNT) {
    //     data.entries.insert(CowBytes::from(number.to_string().as_bytes()),
    //         (KeyInfo { storage_preference: StoragePreference::new(1) },  SlicedCowBytes::from(value.clone())));
    // }

    let start_time = std::time::Instant::now();

    let mut serializer = AllocSerializer::<0>::default();
    serializer.serialize_value(&data).unwrap();
    let bytes = serializer.into_serializer().into_inner();

    let archived = unsafe { rkyv::archived_root::<Type_A>(&bytes[..]) };

    for entry in archived.entries.iter() {
        cloned_data.entries.insert(CowBytes::from(&entry.key) /*CowBytes::from(entry.key.inner.to_vec())*/,
        ( (&entry.value.0).into(),  SlicedCowBytes::from(CowBytes::from(entry.value.1.data.inner.to_vec()))));
        break;
        //break;
            //entry.value.deserialize(&mut rkyv::de::deserializers::SharedDeserializeMap::new()).unwrap());

        //cloned_data.entries.insert(entry.key.deserialize(&mut rkyv::de::deserializers::SharedDeserializeMap::new()).unwrap(),
        //    entry.value.deserialize(&mut rkyv::de::deserializers::SharedDeserializeMap::new()).unwrap());
    }

    // let mut total_entries_fetched = 1;
    // for (key, value) in cloned_data.entries.iter() {
    //     total_entries_fetched = total_entries_fetched + 1;
    //     //println!("{:?}", String::from_utf8(value.1.clone().to_vec()).unwrap());
    // }
    // assert_eq!(total_entries_fetched, ENTRIES_COUNT);

    start_time.elapsed().as_millis()
}

fn serde_case() -> u128 {
    let mut data = Type_B {
        entries: BTreeMap::new()
    };

    let mut cloned_data = Type_B {
        entries: BTreeMap::new()
    };

    fill_data_b(&mut data);
    // let value = CowBytes::from(TEXT_FOR_VALUE.as_bytes());
    
    // for number in (1..ENTRIES_COUNT) {
    //     data.entries.insert(CowBytes::from(number.to_string().as_bytes()),
    //         (KeyInfo { storage_preference: StoragePreference::new(1) },  SlicedCowBytes::from(value.clone())));
    // }
    
    let start_time = std::time::Instant::now();

    let mut buf: Vec<u8> = Vec::new();

    //let xs: Vec<u8> = bincode::serialize(&data).unwrap();

    bincode::serialize_into(&mut buf, &data)
    .map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, e)
    });

    let deserialized = bincode::deserialize::<Type_B>(&buf).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, e)
    }).unwrap();

    // for (key, value) in deserialized.entries.iter() {
    //     cloned_data.entries.insert(key.clone(), value.clone());
    //     //break;
    // }

    // let mut total_entries_fetched = 1;
    // for (key, value) in cloned_data.entries.iter() {
    //     total_entries_fetched = total_entries_fetched + 1;
    //     //println!("{:?}", String::from_utf8(value.1.clone().to_vec()).unwrap());
    // }
    // assert_eq!(total_entries_fetched, ENTRIES_COUNT);

    start_time.elapsed().as_millis()
}

fn main() {
    println!("start!");

    println!("Total time for rkyv_case_a_safe:......{} ms", rkyv_case_a_safe());
    println!("Total time for rkyv_case_a_unsafe:....{} ms", rkyv_case_a_unsafe());
    println!("Total time for serde_case:............{} ms", serde_case());
    println!("Total time for rkyv_case_b_safe:......{} ms", rkyv_case_b_safe());
    println!("Total time for rkyv_case_b_unsafe:....{} ms", rkyv_case_b_unsafe());
    println!("Total time for rkyv_case_b_safe (access one entry):......{} ms", rkyv_case_b_safe_access_one_entity());
    println!("Total time for rkyv_case_b_unsafe (access one entry):....{} ms", rkyv_case_b_unsafe_access_one_entity());
}
