use crate::data::{dat, Value};
use crate::error::{err, Error, Result};

use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug, Clone, PartialEq)]
pub struct SizeifyResult {
    pub raw: Vec<u8>,
    pub ident: String,
    pub kind: String,
    pub ked: Value,
    pub version: Version,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeversifyResult {
    pub ident: String,
    pub kind: String,
    pub version: Version,
    pub size: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SniffResult {
    pub ident: String,
    pub kind: String,
    pub version: Version,
    pub size: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
}

#[allow(non_snake_case)]
pub mod Serialage {
    pub const JSON: &str = "JSON";
}

#[allow(non_snake_case)]
pub mod Identage {
    pub const ACDC: &str = "ACDC";
    pub const KERI: &str = "KERI";
}

#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub mod Ilkage {
    pub const icp: &str = "icp";
    pub const rot: &str = "rot";
    pub const ixn: &str = "ixn";
    pub const dip: &str = "dip";
    pub const drt: &str = "drt";
    pub const rct: &str = "rct";
    pub const ksn: &str = "ksn";
    pub const qry: &str = "qry";
    pub const rpy: &str = "rpy";
    pub const exn: &str = "exn";
    pub const pro: &str = "pro";
    pub const bar: &str = "bar";
    pub const vcp: &str = "vcp";
    pub const vrt: &str = "vrt";
    pub const iss: &str = "iss";
    pub const rev: &str = "rev";
    pub const bis: &str = "bis";
    pub const brv: &str = "brv";
}

#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub mod Tierage {
    pub(crate) const min: &str = "min";
    pub const low: &str = "low";
    pub const med: &str = "med";
    pub const high: &str = "high";
}

#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub mod Ids {
    pub const dollar: &str = "$id";
    pub const at: &str = "@id";
    pub const id: &str = "id";
    pub const i: &str = "i";
    pub const d: &str = "d";
    pub const t: &str = "t";
    pub const k: &str = "k";
    pub const n: &str = "n";
    pub const b: &str = "b";
    pub const a: &str = "a";
    pub const s: &str = "s";
    pub const f: &str = "f";
    pub const v: &str = "v";
    pub const kt: &str = "kt";
    pub const nt: &str = "nt";
    pub const di: &str = "di";
}

const REVER_STRING: &str = "(?P<ident>[A-Z]{4})(?P<major>[0-9a-f])(?P<minor>[0-9a-f])(?P<kind>[A-Z]{4})(?P<size>[0-9a-f]{6})_";
const IDENTS: &[&str] = &[Identage::ACDC, Identage::KERI];
const SERIALS: &[&str] = &[Serialage::JSON];
const ILKS: &[&str] = &[
    Ilkage::icp,
    Ilkage::rot,
    Ilkage::ixn,
    Ilkage::dip,
    Ilkage::drt,
    Ilkage::rct,
    Ilkage::ksn,
    Ilkage::qry,
    Ilkage::rpy,
    Ilkage::exn,
    Ilkage::pro,
    Ilkage::bar,
    Ilkage::vcp,
    Ilkage::vrt,
    Ilkage::iss,
    Ilkage::rev,
    Ilkage::bis,
    Ilkage::brv,
];

pub(crate) const DUMMY: u8 = b'#';

pub const CURRENT_VERSION: &Version = &Version { major: 1, minor: 0 };

const MAXIMUM_START_SIZE: usize = 12;
pub(crate) const VERSION_FULL_SIZE: usize = 17;
pub(crate) const MINIMUM_SNIFF_SIZE: usize = MAXIMUM_START_SIZE + VERSION_FULL_SIZE;

pub fn deversify(vs: &str) -> Result<DeversifyResult> {
    lazy_static! {
        static ref REVER: Regex = Regex::new(REVER_STRING).unwrap();
    };

    if REVER.is_match(vs) {
        let ident = REVER.replace_all(vs, "$ident").to_string();
        let major = u8::from_str_radix(&REVER.replace_all(vs, "$major"), 16)?;
        let minor = u8::from_str_radix(&REVER.replace_all(vs, "$minor"), 16)?;
        let kind = REVER.replace_all(vs, "$kind").to_string();
        let size = u32::from_str_radix(&REVER.replace_all(vs, "$size"), 16)?;

        if !IDENTS.contains(&ident.as_str()) {
            return err!(Error::Validation(format!("invalid ident {ident}")));
        }

        if !SERIALS.contains(&kind.as_str()) {
            return err!(Error::Validation(format!("invalid serialization kind {kind}")));
        }

        return Ok(DeversifyResult { ident, kind, version: Version { major, minor }, size });
    }

    err!(Error::Validation(format!("invalid version string {vs}")))
}

pub fn sizeify(ked: &Value, kind: Option<&str>) -> Result<SizeifyResult> {
    lazy_static! {
        static ref REVER: Regex = Regex::new(REVER_STRING).unwrap();
    };

    if !ked.to_map()?.contains_key("v") {
        return err!(Error::Value("missing or empty version string".to_string()));
    }

    let result = deversify(&ked["v"].to_string()?)?;
    if result.version != *CURRENT_VERSION {
        return err!(Error::Value(format!(
            "unsupported version {}.{}",
            result.version.major, result.version.minor
        )));
    }

    let kind = if let Some(kind) = kind { kind.to_string() } else { result.kind };

    if !SERIALS.contains(&kind.as_str()) {
        return err!(Error::Value(format!("invalid serialization kind {kind}")));
    }

    let raw = &dumps(ked, Some(&kind))?;
    let size = raw.len();

    let start = match REVER.shortest_match(&String::from_utf8(raw.clone())?) {
        Some(m) => m - VERSION_FULL_SIZE,
        // unreachable - deversify has been called which ensures this will match
        None => return err!(Error::Value(format!("invalid version string in raw = {raw:?}"))),
    };

    if start > MAXIMUM_START_SIZE {
        return err!(Error::Value(format!(
            "invalid version string in raw = {raw:?} start = {start}"
        )));
    }

    let fore = raw[..start].to_vec();
    let mut back = raw[start + VERSION_FULL_SIZE..].to_vec();
    let vs = versify(Some(&result.ident), Some(&result.version), Some(&kind), Some(size as u32))?;

    let mut raw = fore;
    raw.append(&mut vs.as_bytes().to_vec());
    raw.append(&mut back);

    if raw.len() != size {
        // unreachable as we constructed this
        return err!(Error::Value(format!("malformed version string size, version string = {vs}")));
    }

    let mut ked = ked.clone();
    ked["v"] = dat!(&vs);

    Ok(SizeifyResult { raw, ident: result.ident, kind, ked, version: result.version })
}

pub fn versify(
    ident: Option<&str>,
    version: Option<&Version>,
    kind: Option<&str>,
    size: Option<u32>,
) -> Result<String> {
    let ident = ident.unwrap_or(Identage::KERI);
    let version = version.unwrap_or(CURRENT_VERSION);
    let kind = kind.unwrap_or(Serialage::JSON);
    let size = size.unwrap_or(0);

    if !IDENTS.contains(&ident) {
        return err!(Error::Validation(format!("invalid ident {ident}")));
    }

    if !SERIALS.contains(&kind) {
        return err!(Error::Validation(format!("invalid serialization kind {kind}")));
    }

    Ok(format!(
        "{ident}{major:01x}{minor:01x}{kind}{size:06x}_",
        major = version.major,
        minor = version.minor
    ))
}

pub(crate) fn loads(raw: &[u8], size: Option<u32>, kind: Option<&str>) -> Result<Value> {
    let kind = kind.unwrap_or(Serialage::JSON);

    if let Some(size) = size {
        match kind {
            Serialage::JSON => {
                let v: serde_json::Value =
                    serde_json::from_str(&String::from_utf8(raw[..(size as usize)].to_vec())?)?;
                Ok(Value::from(&v))
            }
            _ => err!(Error::Validation(format!("invalid serialization kind {kind}"))),
        }
    } else {
        match kind {
            Serialage::JSON => {
                let v: serde_json::Value = serde_json::from_str(&String::from_utf8(raw.to_vec())?)?;
                Ok(Value::from(&v))
            }
            _ => err!(Error::Validation(format!("invalid serialization kind {kind}"))),
        }
    }
}

pub(crate) fn dumps(ked: &Value, kind: Option<&str>) -> Result<Vec<u8>> {
    let kind = kind.unwrap_or(Serialage::JSON);
    match kind {
        Serialage::JSON => Ok(ked.to_json()?.as_bytes().to_vec()),
        _ => err!(Error::Value(format!("invalid serialization kind = {kind}"))),
    }
}

pub fn sniff(raw: &[u8]) -> Result<SniffResult> {
    lazy_static! {
        static ref REVER: Regex = Regex::new(REVER_STRING).unwrap();
    };

    if raw.len() < MINIMUM_SNIFF_SIZE {
        return err!(Error::Value(format!(
            "need more bytes than {bytes} to sniff",
            bytes = raw.len()
        )));
    }

    let raw = &String::from_utf8(raw.to_vec())?;
    let start = match REVER.shortest_match(raw) {
        Some(m) => m - VERSION_FULL_SIZE,
        None => return err!(Error::Value(format!("invalid version string in raw = {raw:?}"))),
    };

    if start > MAXIMUM_START_SIZE {
        return err!(Error::Value(format!(
            "invalid version string in raw = {raw:?} start = {start}"
        )));
    }

    let vs = &raw[start..(start + VERSION_FULL_SIZE)];

    let ident = REVER.replace_all(vs, "$ident").to_string();
    let major = u8::from_str_radix(&REVER.replace_all(vs, "$major"), 16)?;
    let minor = u8::from_str_radix(&REVER.replace_all(vs, "$minor"), 16)?;
    let kind = REVER.replace_all(vs, "$kind").to_string();
    let size = u32::from_str_radix(&REVER.replace_all(vs, "$size"), 16)?;
    let version = Version { major, minor };

    if !SERIALS.contains(&kind.as_str()) {
        return err!(Error::Validation(format!("invalid serialization kind {kind}")));
    }

    Ok(SniffResult { ident, kind, version, size })
}

#[cfg(test)]
mod test {
    use crate::core::common;
    use crate::data::dat;
    use rstest::rstest;

    #[test]
    fn loads() {
        let raw = &dat!({}).to_json().unwrap().as_bytes().to_vec();
        assert!(common::loads(raw, None, None).is_ok());
    }

    #[test]
    fn sniff_unhappy_paths() {
        assert!(common::sniff(&[]).is_err()); // minimum 29 octets
        assert!(common::sniff(
            dat!({"v":"version string must be valid!"}).to_json().unwrap().as_bytes()
        )
        .is_err());
        assert!(common::sniff(
            dat!({"i":"needs to start within 12 characters!","v":"KERI10JSON000000_"})
                .to_json()
                .unwrap()
                .as_bytes()
        )
        .is_err());
        assert!(common::sniff(dat!({"v":"KERI10ABCD000000_","confusing but necessary filler":"hmm...maybe a 12 octet magic prefix?"}).to_json().unwrap().as_bytes()).is_err());
        // needs to have a valid serialization kind
    }

    #[test]
    fn loads_unhappy_paths() {
        let raw = &dat!({}).to_json().unwrap().as_bytes().to_vec();
        assert!(common::loads(raw, None, Some("CESR")).is_err());
        assert!(common::loads(raw, Some(1024), Some("CESR")).is_err());
    }

    #[test]
    fn sizeify_unhappy_paths() {
        assert!(common::sizeify(&dat!({}), None).is_err());
        assert!(common::sizeify(&dat!({"v":"KERIffJSON000000_"}), None).is_err());
        assert!(common::sizeify(&dat!({"v":"KERI10JSON000000_"}), Some("CESR")).is_err());
        assert!(common::sizeify(&dat!({"i":"filler entry","v":"KERI10JSON000000_"}), None).is_err());
    }

    #[test]
    fn versify_unhappy_paths() {
        assert!(common::versify(Some("CESR"), None, None, None).is_err());
        assert!(common::versify(None, None, Some("CESR"), None).is_err());
    }

    #[rstest]
    fn deversify_unhappy_paths(
        #[values("CESR10JSON000000_", "KERI10CESR000000_", "KERIXXJSON000000_")] vs: &str,
    ) {
        assert!(common::deversify(vs).is_err());
    }

    #[test]
    fn dumps_unhappy_paths() {
        assert!(common::dumps(&dat!({}), Some("CESR")).is_err());
    }
}
