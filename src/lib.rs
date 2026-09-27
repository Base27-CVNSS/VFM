//! VFM 1.3 Core Freeze Candidate — compact reference reader/writer.
//! This crate is a conformance seed, not a complete domain-profile SDK.

use sha2::{Digest, Sha256};
use std::{collections::{BTreeMap, BTreeSet}, fmt};

pub const MAGIC:[u8;8]=[0x56,0x46,0x4d,0x00,0x0d,0x0a,0x1a,0x0a];
pub const HEADER_SIZE:usize=256;
pub const DIR_SIZE:usize=64;
pub const CHUNK_SIZE:usize=128;
pub const HASH_SIZE:usize=48;
pub const NONE_HASH_REF:u32=0xffff_ffff;
pub const DATASET_UUID:[u8;16]=[0x00,0x11,0x22,0x33,0x44,0x55,0x66,0x77,0x88,0x99,0xaa,0xbb,0xcc,0xdd,0xee,0xff];

const HF_HASH:u32=1<<1; const HF_PROF:u32=1<<2; const HF_DET:u32=1<<5; const HF_STRICT:u32=1<<6; const HF_IMMUTABLE:u32=1<<7; const HF_MASK:u32=0xff;
const SF_CRITICAL:u32=1<<0; const SF_CONTENT:u32=1<<1; const SF_HOT:u32=1<<2; const SF_CHUNKED:u32=1<<3; const SF_COMPRESSED:u32=1<<5; const SF_ENCRYPTED:u32=1<<6; const SF_IMMUTABLE:u32=1<<7; const SF_MASK:u32=0x3ff;
const CF_CONTENT:u32=1<<0; const CF_COMPRESSED:u32=1<<1; const CF_ENCRYPTED:u32=1<<2; const CF_MASK:u32=0x1f;

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum ErrorCode{E0001,E0002,E0003,E0004,E0005,E0006,E0010,E0011,E0012,E0013,E0014,E0015,E0020,E0021,E0022,E0023,E0030,E0031,E0032,E0033,E0034,E0040}
impl ErrorCode{pub fn as_str(self)->&'static str{match self{Self::E0001=>"E0001",Self::E0002=>"E0002",Self::E0003=>"E0003",Self::E0004=>"E0004",Self::E0005=>"E0005",Self::E0006=>"E0006",Self::E0010=>"E0010",Self::E0011=>"E0011",Self::E0012=>"E0012",Self::E0013=>"E0013",Self::E0014=>"E0014",Self::E0015=>"E0015",Self::E0020=>"E0020",Self::E0021=>"E0021",Self::E0022=>"E0022",Self::E0023=>"E0023",Self::E0030=>"E0030",Self::E0031=>"E0031",Self::E0032=>"E0032",Self::E0033=>"E0033",Self::E0034=>"E0034",Self::E0040=>"E0040"}}}
#[derive(Debug,Clone)] pub struct VfmError{pub code:ErrorCode,pub message:String}
impl VfmError{fn new(code:ErrorCode,msg:impl Into<String>)->Self{Self{code,message:msg.into()}}}
impl fmt::Display for VfmError{fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{write!(f,"{} {}",self.code.as_str(),self.message)}}
impl std::error::Error for VfmError{}

#[derive(Debug,Clone)] pub struct Header{pub format_major:u16,pub format_minor:u16,pub flags:u32,pub section_count:u32,pub directory_offset:u64,pub directory_length:u64,pub manifest_offset:u64,pub manifest_length:u64,pub declared_file_size:u64,pub dataset_uuid:[u8;16],pub root:[u8;32],pub directory_digest:[u8;32],pub manifest_digest:[u8;32]}
#[derive(Debug,Clone)] pub struct DirectoryEntry{pub typ:[u8;4],pub flags:u32,pub logical_id:u64,pub offset:u64,pub stored_length:u64,pub raw_length:u64,pub profile_id:u16,pub section_version:u16,pub compression_id:u16,pub encryption_id:u16,pub crc32c:u32,pub hash_ref:u32,pub aux:u64}
impl DirectoryEntry{pub fn fourcc(&self)->String{String::from_utf8_lossy(&self.typ).into_owned()}}
#[derive(Debug,Clone)] struct Chunk{chunk_id:u64,parent:u64,offset:u64,stored:u64,raw:u64,flags:u32,crc:u32,compression:u16,encryption:u16,hash_ref:u32}
#[derive(Debug,Clone)] struct HRec{kind:u8,id:u64,digest:[u8;32]}
#[derive(Debug,Clone)] pub struct VfmFile{pub header:Header,pub directory:Vec<DirectoryEntry>,pub profiles:Vec<String>,pub warnings:Vec<ErrorCode>}

fn u16le(b:&[u8],o:usize)->u16{u16::from_le_bytes(b[o..o+2].try_into().unwrap())}
fn u32le(b:&[u8],o:usize)->u32{u32::from_le_bytes(b[o..o+4].try_into().unwrap())}
fn u64le(b:&[u8],o:usize)->u64{u64::from_le_bytes(b[o..o+8].try_into().unwrap())}
fn w16(b:&mut[u8],o:usize,v:u16){b[o..o+2].copy_from_slice(&v.to_le_bytes())}
fn w32(b:&mut[u8],o:usize,v:u32){b[o..o+4].copy_from_slice(&v.to_le_bytes())}
fn w64(b:&mut[u8],o:usize,v:u64){b[o..o+8].copy_from_slice(&v.to_le_bytes())}
pub fn sha256(b:&[u8])->[u8;32]{let mut h=Sha256::new();h.update(b);h.finalize().into()}
pub fn crc32c_value(b:&[u8])->u32{crc32c::crc32c(b)}
fn slice(b:&[u8],o:u64,n:u64)->Result<&[u8],VfmError>{let e=o.checked_add(n).ok_or_else(||VfmError::new(ErrorCode::E0010,"range overflow"))?;if e>b.len() as u64{return Err(VfmError::new(ErrorCode::E0010,"range outside artifact"))}Ok(&b[o as usize..e as usize])}

fn parse_header(b:&[u8])->Result<Header,VfmError>{
 if b.len()<256{return Err(VfmError::new(ErrorCode::E0010,"short Header"))}
 if b[..8]!=MAGIC{return Err(VfmError::new(ErrorCode::E0001,"BAD_MAGIC"))}
 if u16le(b,8)!=1{return Err(VfmError::new(ErrorCode::E0002,"UNSUPPORTED_MAJOR"))}
 if u16le(b,12)!=256{return Err(VfmError::new(ErrorCode::E0003,"BAD_HEADER_SIZE"))}
 if b[16]!=1{return Err(VfmError::new(ErrorCode::E0004,"BAD_BYTE_ORDER"))}
 if crc32c_value(&b[..252])!=u32le(b,252){return Err(VfmError::new(ErrorCode::E0005,"HEADER_CRC_MISMATCH"))}
 let flags=u32le(b,20);if flags&!HF_MASK!=0||u32le(b,28)!=0||u64le(b,232)!=0||b[240..252].iter().any(|&x|x!=0){return Err(VfmError::new(ErrorCode::E0006,"RESERVED_NONZERO"))}
 if u16le(b,14)!=64||b[17]!=1||b[18]!=1||b[19]!=1{return Err(VfmError::new(ErrorCode::E0006,"frozen Header field mismatch"))}
 if flags&HF_DET!=0&&(b[208..224].iter().any(|&x|x!=0)||u64le(b,224)!=0){return Err(VfmError::new(ErrorCode::E0006,"deterministic build contains entropy"))}
 if u64le(b,72)!=0||u64le(b,104)!=0{return Err(VfmError::new(ErrorCode::E0006,"footer/generation not Core 1.3"))}
 let mut uuid=[0;16];uuid.copy_from_slice(&b[88..104]);if uuid.iter().all(|&x|x==0){return Err(VfmError::new(ErrorCode::E0006,"zero dataset_uuid"))}
 let mut root=[0;32];root.copy_from_slice(&b[112..144]);let mut dd=[0;32];dd.copy_from_slice(&b[144..176]);let mut md=[0;32];md.copy_from_slice(&b[176..208]);
 Ok(Header{format_major:1,format_minor:u16le(b,10),flags,section_count:u32le(b,24),directory_offset:u64le(b,32),directory_length:u64le(b,40),manifest_offset:u64le(b,48),manifest_length:u64le(b,56),declared_file_size:u64le(b,80),dataset_uuid:uuid,root,directory_digest:dd,manifest_digest:md})
}
fn parse_dir(r:&[u8])->Result<DirectoryEntry,VfmError>{let mut typ=[0;4];typ.copy_from_slice(&r[..4]);let flags=u32le(r,4);if flags&!SF_MASK!=0{return Err(VfmError::new(ErrorCode::E0006,"reserved section flag"))}Ok(DirectoryEntry{typ,flags,logical_id:u64le(r,8),offset:u64le(r,16),stored_length:u64le(r,24),raw_length:u64le(r,32),profile_id:u16le(r,40),section_version:u16le(r,42),compression_id:u16le(r,44),encryption_id:u16le(r,46),crc32c:u32le(r,48),hash_ref:u32le(r,52),aux:u64le(r,56)})}
fn parse_chunk(r:&[u8])->Result<Chunk,VfmError>{let flags=u32le(r,56);if flags&!CF_MASK!=0||r[96..128].iter().any(|&x|x!=0){return Err(VfmError::new(ErrorCode::E0006,"reserved ChunkDescriptor field"))}let c=Chunk{chunk_id:u64le(r,0),parent:u64le(r,8),offset:u64le(r,32),stored:u64le(r,40),raw:u64le(r,48),flags,crc:u32le(r,60),compression:u16le(r,64),encryption:u16le(r,66),hash_ref:u32le(r,68)};if c.chunk_id==0{return Err(VfmError::new(ErrorCode::E0013,"zero chunk_id"))}if c.offset%8!=0{return Err(VfmError::new(ErrorCode::E0015,"unaligned chunk"))}if (flags&CF_COMPRESSED!=0)!=(c.compression!=0)||(flags&CF_ENCRYPTED!=0)!=(c.encryption!=0){return Err(VfmError::new(ErrorCode::E0006,"chunk codec flag/id mismatch"))}Ok(c)}
fn parse_hash(b:&[u8])->Result<Vec<HRec>,VfmError>{if b.len()%48!=0{return Err(VfmError::new(ErrorCode::E0034,"bad HASH length"))}let mut out=Vec::new();let mut prev=None;for r in b.chunks_exact(48){if !(r[0]==1||r[0]==2)||r[1]!=1||r[2]!=32||r[3]!=3||u32le(r,44)!=0{return Err(VfmError::new(ErrorCode::E0034,"bad HashRecord"))}let key=(r[0],u64le(r,4));if prev.is_some_and(|p|p>=key){return Err(VfmError::new(ErrorCode::E0034,"HASH not strictly sorted"))}prev=Some(key);let mut d=[0;32];d.copy_from_slice(&r[12..44]);out.push(HRec{kind:key.0,id:key.1,digest:d})}Ok(out)}

fn cbor_len(b:&[u8],p:&mut usize,major:u8)->Result<usize,VfmError>{if *p>=b.len(){return Err(VfmError::new(ErrorCode::E0031,"truncated CBOR"))}let x=b[*p];*p+=1;if x>>5!=major{return Err(VfmError::new(ErrorCode::E0031,"CBOR major type"))}match x&31{n@0..=23=>Ok(n as usize),24=>{if *p>=b.len(){return Err(VfmError::new(ErrorCode::E0031,"truncated CBOR"))}let v=b[*p]as usize;*p+=1;Ok(v)},_=>Err(VfmError::new(ErrorCode::E0031,"fixture reader requires short deterministic CBOR"))}}
fn cbor_uint(b:&[u8],p:&mut usize)->Result<u64,VfmError>{Ok(cbor_len(b,p,0)? as u64)}
fn validate_meta(b:&[u8],uuid:&[u8;16])->Result<(),VfmError>{let mut p=0;let n=cbor_len(b,&mut p,5)?;let mut need=[false;4];let mut prev=None;for _ in 0..n{let k=cbor_uint(b,&mut p)?;if prev.is_some_and(|x|x>=k){return Err(VfmError::new(ErrorCode::E0031,"non-deterministic META keys"))}prev=Some(k);match k{0=>{if cbor_len(b,&mut p,4)?!=2||cbor_uint(b,&mut p)?!=1||cbor_uint(b,&mut p)?!=3{return Err(VfmError::new(ErrorCode::E0031,"META format"))}need[0]=true},1=>{if cbor_len(b,&mut p,2)?!=16||p+16>b.len()||b[p..p+16]!=uuid[..]{return Err(VfmError::new(ErrorCode::E0031,"META uuid"))}p+=16;need[1]=true},4=>{if cbor_uint(b,&mut p)?!=3{return Err(VfmError::new(ErrorCode::E0031,"META profile id"))}need[2]=true},5=>{if cbor_uint(b,&mut p)?!=2{return Err(VfmError::new(ErrorCode::E0031,"META hash id"))}need[3]=true},_=>return Err(VfmError::new(ErrorCode::E0031,"reference seed only accepts frozen required META keys"))}}if p!=b.len()||need.iter().any(|&x|!x){return Err(VfmError::new(ErrorCode::E0031,"META incomplete"))}Ok(())}
fn profiles(b:&[u8])->Result<Vec<String>,VfmError>{let mut p=0;let n=cbor_len(b,&mut p,4)?;let mut v=Vec::new();for _ in 0..n{let l=cbor_len(b,&mut p,3)?;if p+l>b.len(){return Err(VfmError::new(ErrorCode::E0031,"PROF truncated"))}v.push(std::str::from_utf8(&b[p..p+l]).map_err(|_|VfmError::new(ErrorCode::E0031,"PROF utf8"))?.to_owned());p+=l}if p!=b.len(){return Err(VfmError::new(ErrorCode::E0031,"PROF trailing bytes"))}Ok(v)}

pub fn read(b:&[u8])->Result<VfmFile,VfmError>{
 let h=parse_header(b)?;if h.directory_offset!=256{return Err(VfmError::new(ErrorCode::E0010,"directory_offset"))}let need=h.section_count as u64*64;if h.directory_length!=need{return Err(VfmError::new(ErrorCode::E0011,"DIRECTORY_LENGTH_MISMATCH"))}let dend=256u64+need;if dend>16384||dend>b.len()as u64{return Err(VfmError::new(ErrorCode::E0010,"DIRECTORY_OUT_OF_BOUNDS"))}if h.declared_file_size!=b.len()as u64{return Err(VfmError::new(ErrorCode::E0010,"declared_file_size"))}
 let db=slice(b,256,need)?;if sha256(db)!=h.directory_digest{return Err(VfmError::new(ErrorCode::E0012,"DIRECTORY_DIGEST_MISMATCH"))}let mut dir=Vec::new();let mut ids=BTreeSet::new();for r in db.chunks_exact(64){let e=parse_dir(r)?;if e.logical_id==0||!ids.insert(e.logical_id){return Err(VfmError::new(ErrorCode::E0013,"DUPLICATE_LOGICAL_ID"))}if e.offset%8!=0{return Err(VfmError::new(ErrorCode::E0015,"UNALIGNED_OFFSET"))}if e.offset<dend&&e.stored_length>0{return Err(VfmError::new(ErrorCode::E0014,"overlap Header/Directory"))}slice(b,e.offset,e.stored_length)?;if e.flags&SF_CHUNKED!=0{if e.stored_length!=e.aux*128||e.compression_id!=0||e.encryption_id!=0||e.hash_ref!=NONE_HASH_REF||e.flags&(SF_COMPRESSED|SF_ENCRYPTED)!=0{return Err(VfmError::new(ErrorCode::E0006,"bad CHUNKED entry"))}}else if (e.flags&SF_COMPRESSED!=0)!=(e.compression_id!=0)||(e.flags&SF_ENCRYPTED!=0)!=(e.encryption_id!=0){return Err(VfmError::new(ErrorCode::E0006,"section codec flag/id"))}dir.push(e)}
 let mut ranges:Vec<(u64,u64)>=dir.iter().filter(|e|e.stored_length>0).map(|e|(e.offset,e.offset+e.stored_length)).collect();ranges.sort_unstable();for w in ranges.windows(2){if w[0].1>w[1].0{return Err(VfmError::new(ErrorCode::E0014,"OVERLAPPING_RANGES"))}}
 let by:BTreeMap<u64,&DirectoryEntry>=dir.iter().map(|e|(e.logical_id,e)).collect();for(id,t)in[(1,*b"META"),(2,*b"HASH"),(3,*b"PROF")]{let e=by.get(&id).ok_or_else(||VfmError::new(ErrorCode::E0020,"required Core section missing"))?;if e.typ!=t||e.profile_id!=0||e.section_version!=1||e.flags&SF_CRITICAL==0{return Err(VfmError::new(ErrorCode::E0020,"required Core section mismatch"))}}
 let mut warnings=Vec::new();for e in&dir{if e.profile_id==0&&!(e.typ==*b"META"||e.typ==*b"HASH"||e.typ==*b"PROF"){if e.flags&SF_CRITICAL!=0{return Err(VfmError::new(ErrorCode::E0020,"UNKNOWN_CRITICAL_SECTION"))}warnings.push(ErrorCode::E0021)}}
 for e in&dir{let s=slice(b,e.offset,e.stored_length)?;if crc32c_value(s)!=e.crc32c{return Err(VfmError::new(ErrorCode::E0030,"STORED_CRC_MISMATCH"))}if e.flags&SF_CHUNKED==0{if e.compression_id!=0{return Err(VfmError::new(ErrorCode::E0022,"UNSUPPORTED_COMPRESSION"))}if e.encryption_id!=0{return Err(VfmError::new(ErrorCode::E0023,"UNSUPPORTED_ENCRYPTION"))}if e.stored_length!=e.raw_length{return Err(VfmError::new(ErrorCode::E0031,"RAW_LENGTH_MISMATCH"))}}}
 let mut chunks=Vec::new();let mut cids=BTreeSet::new();let mut all=ranges;for e in&dir{if e.flags&SF_CHUNKED==0{continue}let table=slice(b,e.offset,e.stored_length)?;let mut sum=0u64;for r in table.chunks_exact(128){let c=parse_chunk(r)?;if c.parent!=e.logical_id||!cids.insert(c.chunk_id){return Err(VfmError::new(ErrorCode::E0013,"chunk id/parent"))}let end=c.offset.checked_add(c.stored).ok_or_else(||VfmError::new(ErrorCode::E0010,"chunk overflow"))?;let s=slice(b,c.offset,c.stored)?;if crc32c_value(s)!=c.crc{return Err(VfmError::new(ErrorCode::E0030,"chunk CRC"))}if c.compression!=0{return Err(VfmError::new(ErrorCode::E0022,"chunk compression"))}if c.encryption!=0{return Err(VfmError::new(ErrorCode::E0023,"chunk encryption"))}if c.stored!=c.raw{return Err(VfmError::new(ErrorCode::E0031,"chunk raw length"))}sum=sum.checked_add(c.raw).ok_or_else(||VfmError::new(ErrorCode::E0040,"length sum"))?;all.push((c.offset,end));chunks.push(c)}if sum!=e.raw_length{return Err(VfmError::new(ErrorCode::E0031,"chunked raw sum"))}}all.sort_unstable();for w in all.windows(2){if w[0].1>w[1].0{return Err(VfmError::new(ErrorCode::E0014,"OVERLAPPING_RANGES"))}}
 let meta=by[&1];if h.manifest_offset!=meta.offset||h.manifest_length!=meta.raw_length{return Err(VfmError::new(ErrorCode::E0010,"manifest location"))}let mb=slice(b,meta.offset,meta.raw_length)?;if sha256(mb)!=h.manifest_digest{return Err(VfmError::new(ErrorCode::E0033,"manifest digest"))}
 let he=by[&2];let hb=slice(b,he.offset,he.raw_length)?;if sha256(hb)!=h.root{return Err(VfmError::new(ErrorCode::E0034,"ROOT_DIGEST_MISMATCH"))}let hr=parse_hash(hb)?;
 for e in&dir{if e.flags&SF_CONTENT!=0&&e.flags&SF_CHUNKED==0{let r=hr.get(e.hash_ref as usize).ok_or_else(||VfmError::new(ErrorCode::E0032,"HASH_REF_OUT_OF_RANGE"))?;if r.kind!=1||r.id!=e.logical_id{return Err(VfmError::new(ErrorCode::E0032,"wrong hash object"))}if sha256(slice(b,e.offset,e.raw_length)?)!=r.digest{return Err(VfmError::new(ErrorCode::E0033,"CONTENT_HASH_MISMATCH"))}}
 for c in&chunks{if c.flags&CF_CONTENT!=0{let r=hr.get(c.hash_ref as usize).ok_or_else(||VfmError::new(ErrorCode::E0032,"chunk HASH_REF"))?;if r.kind!=2||r.id!=c.chunk_id{return Err(VfmError::new(ErrorCode::E0032,"wrong chunk hash object"))}if sha256(slice(b,c.offset,c.raw)?)!=r.digest{return Err(VfmError::new(ErrorCode::E0033,"chunk CONTENT_HASH_MISMATCH"))}}
 validate_meta(mb,&h.dataset_uuid)?;let pe=by[&3];let ps=profiles(slice(b,pe.offset,pe.raw_length)?)?;for e in&dir{if e.profile_id==65535{return Err(VfmError::new(ErrorCode::E0006,"reserved profile id"))}if e.profile_id!=0&&e.profile_id as usize>ps.len(){return Err(VfmError::new(ErrorCode::E0020,"profile id range"))}}
 Ok(VfmFile{header:h,directory:dir,profiles:ps,warnings})
}

fn align(v:usize,a:usize)->usize{(v+a-1)&!(a-1)}
fn cbor_uint(n:u64)->Vec<u8>{match n{0..=23=>vec![n as u8],24..=255=>vec![0x18,n as u8],_=>unimplemented!("minimal fixture uses short CBOR integers")}}
fn cbor_bstr(b:&[u8])->Vec<u8>{assert!(b.len()<24);let mut v=vec![0x40|b.len()as u8];v.extend_from_slice(b);v}
fn cbor_array(x:&[Vec<u8>])->Vec<u8>{assert!(x.len()<24);let mut v=vec![0x80|x.len()as u8];for i in x{v.extend_from_slice(i)}v}
fn cbor_map(mut x:Vec<(u64,Vec<u8>)>)->Vec<u8>{x.sort_by_key(|x|x.0);let mut v=vec![0xa0|x.len()as u8];for(k,a)in x{v.extend(cbor_uint(k));v.extend(a)}v}
fn meta()->Vec<u8>{cbor_map(vec![(0,cbor_array(&[cbor_uint(1),cbor_uint(3)])),(1,cbor_bstr(&DATASET_UUID)),(4,cbor_uint(3)),(5,cbor_uint(2))])}
fn hrec(id:u64,d:[u8;32])->[u8;48]{let mut o=[0;48];o[0]=1;o[1]=1;o[2]=32;o[3]=3;w64(&mut o,4,id);o[12..44].copy_from_slice(&d);o}
fn dent(t:[u8;4],flags:u32,id:u64,off:u64,len:u64,crc:u32,href:u32)->[u8;64]{let mut o=[0;64];o[..4].copy_from_slice(&t);w32(&mut o,4,flags);w64(&mut o,8,id);w64(&mut o,16,off);w64(&mut o,24,len);w64(&mut o,32,len);w16(&mut o,42,1);w32(&mut o,48,crc);w32(&mut o,52,href);o}

pub fn write_core_minimal()->Vec<u8>{
 let m=meta();let p=cbor_array(&[]);let mut hp=Vec::new();hp.extend_from_slice(&hrec(1,sha256(&m)));hp.extend_from_slice(&hrec(3,sha256(&p)));let root=sha256(&hp);let mo=448usize;let po=align(mo+m.len(),8);let ho=align(po+p.len(),8);let size=ho+hp.len();let me=dent(*b"META",SF_CRITICAL|SF_CONTENT|SF_HOT|SF_IMMUTABLE,1,mo as u64,m.len()as u64,crc32c_value(&m),0);let he=dent(*b"HASH",SF_CRITICAL|SF_IMMUTABLE,2,ho as u64,hp.len()as u64,crc32c_value(&hp),NONE_HASH_REF);let pe=dent(*b"PROF",SF_CRITICAL|SF_CONTENT|SF_HOT|SF_IMMUTABLE,3,po as u64,p.len()as u64,crc32c_value(&p),1);let mut d=Vec::new();d.extend_from_slice(&me);d.extend_from_slice(&he);d.extend_from_slice(&pe);
 let mut h=[0;256];h[..8].copy_from_slice(&MAGIC);w16(&mut h,8,1);w16(&mut h,10,3);w16(&mut h,12,256);w16(&mut h,14,64);h[16]=1;h[17]=1;h[18]=1;h[19]=1;w32(&mut h,20,HF_HASH|HF_PROF|HF_DET|HF_STRICT|HF_IMMUTABLE);w32(&mut h,24,3);w64(&mut h,32,256);w64(&mut h,40,192);w64(&mut h,48,mo as u64);w64(&mut h,56,m.len()as u64);w64(&mut h,80,size as u64);h[88..104].copy_from_slice(&DATASET_UUID);h[112..144].copy_from_slice(&root);h[144..176].copy_from_slice(&sha256(&d));h[176..208].copy_from_slice(&sha256(&m));let hc=crc32c_value(&h[..252]);w32(&mut h,252,hc);
 let mut out=vec![0;size];out[..256].copy_from_slice(&h);out[256..448].copy_from_slice(&d);out[mo..mo+m.len()].copy_from_slice(&m);out[po..po+p.len()].copy_from_slice(&p);out[ho..ho+hp.len()].copy_from_slice(&hp);out
}
pub fn artifact_sha256(b:&[u8])->String{use std::fmt::Write as _;let d=sha256(b);let mut s=String::with_capacity(64);for x in d{write!(&mut s,"{x:02x}").unwrap()}s}
