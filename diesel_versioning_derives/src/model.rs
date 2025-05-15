use proc_macro_error::abort_call_site;
use proc_macro2::Ident;
use syn::{
    Data, DataStruct, Fields, FieldsNamed, FieldsUnnamed, Path, Result, TypePath,
    parenthesized,
    parse::{Parse, ParseStream, Peek},
    punctuated::Punctuated,
    token::{Comma, Eq},
};

pub enum StructAttr {
    TableName(Path),
    CheckForBackend(syn::punctuated::Punctuated<TypePath, syn::Token![,]>),
}

impl Parse for StructAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        let name_str = name.to_string();
        match &*name_str {
            "table_name" => Ok(StructAttr::TableName(parse_eq(input)?)),
            "check_for_backend" => Ok(StructAttr::CheckForBackend(parse_paren_list(
                input,
                syn::Token![,],
            )?)),
            _ => Err(syn::Error::new(name.span(), "uninteressting attribute")),
        }
    }
}

pub fn parse_eq<T: Parse>(input: ParseStream) -> Result<T> {
    if input.is_empty() {
        return Err(syn::Error::new(
            input.span(),
            "unexpected end of input, expected `=`".to_string(),
        ));
    }

    input.parse::<Eq>()?;
    input.parse()
}

pub fn parse_paren_list<T, D>(
    input: ParseStream,
    sep: D,
) -> Result<syn::punctuated::Punctuated<T, <D as Peek>::Token>>
where
    T: Parse,
    D: Peek,
    D::Token: Parse,
{
    if input.is_empty() {
        return Err(syn::Error::new(
            input.span(),
            "unexpected end of input, expected parentheses".to_string(),
        ));
    }

    let content;
    parenthesized!(content in input);
    content.parse_terminated(T::parse, sep)
}

pub struct Model {
    version_fieldname: Option<Ident>,
    table_names: Vec<Path>,
    backends: Vec<Path>,
}
impl Model {
    pub(crate) fn from_item(item: &syn::DeriveInput) -> Result<Self> {
        // extract fields of struct
        let fields = match &item.data {
            Data::Struct(DataStruct {
                fields: Fields::Named(FieldsNamed { named, .. }),
                ..
            }) => Some(named),
            Data::Struct(DataStruct {
                fields: Fields::Unnamed(FieldsUnnamed { unnamed, .. }),
                ..
            }) => Some(unnamed),
            _ => None,
        };
        let version_fieldname = match fields {
            Some(fields) => {
                let version_fields: Vec<_> = fields
                    .iter()
                    .filter(|f| {
                        if f.attrs
                            .iter()
                            .filter(|attr| attr.path().is_ident("version"))
                            .count()
                            == 1
                        {
                            return true;
                        }
                        false
                    })
                    .collect();
                match version_fields.len() {
                    0 => None,
                    1 => version_fields
                        .first()
                        .expect("first element on non empty list")
                        .ident
                        .to_owned(),
                    2.. => abort_call_site!("only one version field is supported"),
                }
            }
            None => None,
        };
        //parse attributes
        let mut table_names = Vec::new();
        let mut backends = Vec::new();
        let attrs = &item.attrs;
        for attr in attrs {
            if attr.meta.path().is_ident("diesel") {
                let map = attr.parse_args_with(Punctuated::<StructAttr, Comma>::parse_terminated);
                if let Ok(map) = map {
                    for attr in map.into_iter() {
                        match attr {
                            StructAttr::TableName(path) => {
                                table_names.push(path);
                            }
                            StructAttr::CheckForBackend(path) => {
                                backends.extend(path.into_iter().map(|backend| backend.path));
                            }
                        }
                    }
                }
            }
        }
        Ok(Self {
            version_fieldname,
            table_names,
            backends,
        })
    }
    pub(crate) fn version_fieldname(&self) -> &Option<Ident> {
        &self.version_fieldname
    }

    pub(crate) fn table_names(&self) -> &[Path] {
        match self.table_names.len() {
            0 => abort_call_site!("exptected table_name, but didn't found one"),
            _ => &self.table_names,
        }
    }
    pub(crate) fn backends(&self) -> &[Path] {
        match self.backends.len() {
            0 => abort_call_site!("exptected backend, but didn't found one"),
            _ => &self.backends,
        }
    }
}
