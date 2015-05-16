//!
//! ```
//! use sxd_document::Package;
//! let package = Package::new();
//! let doc = package.as_document();
//!
//! let hello = doc.create_element("hello");
//! hello.set_attribute_value("planet", "Earth");
//! let comment = doc.create_comment("What about other planets?");
//! let text = doc.create_text("Greetings, Earthlings!");
//!
//! hello.append_child(comment);
//! hello.append_child(text);
//! doc.root().append_child(hello);
//! ```
//!
//! ### Memory and ownership
//!
//! The `Package` struct is responsible for owning every node in the
//! document. Strings are interned, allowing repeated text to consume
//! less memory. This is very useful for documents containing lots of
//! the same attributes and tag names.
//!
//! The flip side of this decision is that allocated nodes and strings
//! are not deallocated until the entire `Package` is dropped. This is
//! a reasonable decision for two common cases: building up an XML
//! document and reading an XML document. You may wish to perform
//! large modifications to your data *before* creating a document.
//!
//! ### Namespaces, QNames, and Prefixes
//!
//! The names of elements and attributes may use namespaces. XML
//! namespacing uses URIs to uniquely distinguish items with the same
//! local name. A qualified name (`QName`) combines this optional URI
//! with the local name.
//!
//! When an XML document is represented as text, namespaces are given
//! a shorthand reference known as a prefix. Prefix names are
//! non-authoritative, and only the URI can be used to namespace a
//! name.
//!
//! Elements and attributes may specify a *preferred prefix*, which is
//! an indication of what the user would like to be used as a
//! prefix. There are times where the preferred prefix would cause a
//! conflict, and so an autogenerated prefix will be used instead.
//!
//! ### Design decisions
//!
//! Try to leverage the type system as much as possible.

#![cfg_attr(feature = "unstable", feature(core))]
#![cfg_attr(feature = "unstable", feature(test))]

extern crate typed_arena;

#[macro_use]
extern crate peresil;

use std::fmt;

mod str_ext;
mod string_pool;
mod raw;
mod str;

#[doc(hidden)]
pub mod thindom;
pub mod dom;
pub mod parser;
pub mod writer;

pub use str::XmlChar;

/// A prefixed name. This represents what is found in the string form
/// of an XML document, and does not apply any namespace mapping.
#[derive(Debug,Copy,Clone,PartialEq,Eq,PartialOrd,Ord)]
pub struct PrefixedName<'a> {
    prefix: Option<&'a str>,
    local_part: &'a str,
}

impl<'a> PrefixedName<'a> {
    /// Create a `PrefixedName` without a prefix
    pub fn new(local_part: &str) -> PrefixedName {
        PrefixedName::with_prefix(None, local_part)
    }

    /// Create a `PrefixedName` without an optional prefix
    pub fn with_prefix(prefix: Option<&'a str>, local_part: &'a str) -> PrefixedName<'a> {
        PrefixedName {
            prefix: prefix,
            local_part: local_part,
        }
    }

    pub fn prefix(&self) -> Option<&str> { self.prefix }
    pub fn local_part(&self) -> &str { self.local_part }
}

/// A namespace-qualified name. This represents the name of an element
/// or attribute *after* the prefix has been mapped to a specific
/// namespace.
#[derive(Debug,Copy,Clone,PartialEq)]
pub struct QName<'s> {
    namespace_uri: Option<&'s str>,
    local_part: &'s str,
}

impl<'s> QName<'s> {
    /// Create a `QName` without a namespace
    pub fn new(local_part: &'s str) -> QName<'s> {
        QName::with_namespace_uri(None, local_part)
    }

    /// Create a `QName` with an optional namespace
    pub fn with_namespace_uri(namespace_uri: Option<&'s str>, local_part: &'s str) -> QName<'s> {
        QName {
            namespace_uri: namespace_uri,
            local_part: local_part,
        }
    }

    pub fn namespace_uri(&self) -> Option<&'s str> { self.namespace_uri }
    pub fn local_part(&self) -> &'s str { self.local_part }
}

impl<'s> From<(&'s str, &'s str)> for QName<'s> {
    fn from(v: (&'s str, &'s str)) -> QName<'s> {
        QName { namespace_uri: Some(v.0), local_part: v.1 }
    }
}

impl<'s> From<&'s str> for QName<'s> {
    fn from(v: &'s str) -> QName<'s> {
        QName { namespace_uri: None, local_part: v }
    }
}

/// The main entrypoint to an XML document
///
/// This is an opaque structure that stores the internal details of
/// the XML document. Modify the document via `as_document`.
pub struct Package {
    storage: raw::Storage,
    connections: raw::Connections,
}

impl Package {
    pub fn new() -> Package {
        let s = raw::Storage::new();
        let root = s.create_root();
        Package {
            storage: s,
            connections: raw::Connections::new(root),
        }
    }

    pub fn as_document(&self) -> dom::Document {
        dom::Document::new(&self.storage, &self.connections)
    }

    #[doc(hidden)]
    pub fn as_thin_document(&self) -> (thindom::Storage, thindom::Connections) {
        let s = thindom::Storage::new(&self.storage);
        let c = thindom::Connections::new(&self.connections);
        (s, c)
    }
}

impl PartialEq for Package {
    fn eq(&self, other: &Package) -> bool {
        self as *const Package == other as *const Package
    }
}

impl fmt::Debug for Package {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Package")
    }
}
