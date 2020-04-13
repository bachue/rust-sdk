//! 资源管理模块
//!
//! 封装与资源管理相关的数据结构

use super::object::Object;
use crate::utils::base64;
use std::collections::HashMap;

pub(super) trait ToURI {
    fn to_uri(&self) -> String;
}

pub(super) struct Stat<'a> {
    object: &'a Object,
}

impl<'a> Stat<'a> {
    #[inline]
    pub(super) fn new(object: &'a Object) -> Self {
        Self { object }
    }
}

impl ToURI for Stat<'_> {
    fn to_uri(&self) -> String {
        "/stat/".to_owned() + self.object.encoded_entry_uri()
    }
}

pub(super) struct Delete<'a> {
    object: &'a Object,
}

impl<'a> Delete<'a> {
    #[inline]
    pub(super) fn new(object: &'a Object) -> Self {
        Self { object }
    }
}

impl ToURI for Delete<'_> {
    fn to_uri(&self) -> String {
        "/delete/".to_owned() + self.object.encoded_entry_uri()
    }
}

pub(super) struct Move<'a> {
    src_object: &'a Object,
    dest_object: &'a Object,
    force: bool,
}

impl<'a> Move<'a> {
    #[inline]
    pub(super) fn new(src: &'a Object, dest: &'a Object, force: bool) -> Self {
        Self {
            src_object: src,
            dest_object: dest,
            force,
        }
    }
}

impl ToURI for Move<'_> {
    fn to_uri(&self) -> String {
        let mut uri = "/move/".to_owned()
            + self.src_object.encoded_entry_uri()
            + "/"
            + self.dest_object.encoded_entry_uri()
            + "/force/";
        if self.force {
            uri.push_str("true");
        } else {
            uri.push_str("false");
        }
        uri
    }
}

pub(super) struct Copy<'a> {
    src_object: &'a Object,
    dest_object: &'a Object,
    force: bool,
}

impl<'a> Copy<'a> {
    #[inline]
    pub(super) fn new(src: &'a Object, dest: &'a Object, force: bool) -> Self {
        Self {
            src_object: src,
            dest_object: dest,
            force,
        }
    }
}

impl ToURI for Copy<'_> {
    fn to_uri(&self) -> String {
        let mut uri = "/copy/".to_owned()
            + self.src_object.encoded_entry_uri()
            + "/"
            + self.dest_object.encoded_entry_uri()
            + "/force/";
        if self.force {
            uri.push_str("true");
        } else {
            uri.push_str("false");
        }
        uri
    }
}

pub(super) struct Chgm<'a> {
    object: &'a Object,
    mime_type: Option<&'a str>,
    metadata: HashMap<&'a str, &'a str>,
}

impl<'a> Chgm<'a> {
    #[inline]
    pub(super) fn new(object: &'a Object, mime_type: Option<&'a str>, metadata: HashMap<&'a str, &'a str>) -> Self {
        Self {
            object,
            mime_type,
            metadata,
        }
    }
}

impl ToURI for Chgm<'_> {
    fn to_uri(&self) -> String {
        let mut uri = "/chgm/".to_owned() + self.object.encoded_entry_uri();
        if let Some(mime_type) = self.mime_type {
            uri.push_str("/mime/");
            uri.push_str(&base64::urlsafe(mime_type.as_bytes()));
        }
        for (metadata_key, metadata_value) in self.metadata.iter() {
            uri.push_str("/x-qn-meta-");
            uri.push_str(metadata_key);
            uri.push_str("/");
            uri.push_str(&base64::urlsafe(metadata_value.as_bytes()));
        }
        uri
    }
}

pub(super) struct SetMeta<'a> {
    object: &'a Object,
    metadata: HashMap<&'a str, &'a str>,
}

impl<'a> SetMeta<'a> {
    #[inline]
    pub(super) fn new(object: &'a Object, metadata: HashMap<&'a str, &'a str>) -> Self {
        Self { object, metadata }
    }
}

impl ToURI for SetMeta<'_> {
    fn to_uri(&self) -> String {
        let mut uri = "/chgm/".to_owned() + self.object.encoded_entry_uri();
        for (metadata_key, metadata_value) in self.metadata.iter() {
            uri.push_str("/x-qn-meta-");
            uri.push_str(metadata_key);
            uri.push_str("/");
            uri.push_str(&base64::urlsafe(metadata_value.as_bytes()));
        }
        uri
    }
}
