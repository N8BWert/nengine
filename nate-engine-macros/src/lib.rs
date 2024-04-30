extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{parse::{Parse, ParseStream, Result}, parse_macro_input, Block, Error, Expr, ExprBinary, FnArg, Ident, ItemFn, ItemStruct, Token};

use std::collections::{HashMap, HashSet};

struct IgnoreArgs {
    ignore_identifiers: HashSet<String>,
}

impl Parse for IgnoreArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut ignore_identifiers = HashSet::new();

        let parts = input.parse_terminated(Expr::parse, Token![,])?;
        for part in parts.iter() {
            if let Expr::Assign(assignment) = part {
                if let Expr::Path(path) = assignment.left.as_ref() {
                    if let Some(segment) = path.path.segments.first() {
                        if segment.ident.to_string().as_str() == "singular" {
                            match assignment.right.as_ref() {
                                Expr::Path(path) => {
                                    if let Some(segment) = path.path.segments.first() {
                                        ignore_identifiers.insert(segment.ident.to_string());
                                    }
                                },
                                Expr::Array(array) => {
                                    for element in array.elems.iter() {
                                        if let Expr::Path(path) = element {
                                            if let Some(segment) = path.path.segments.first() {
                                                ignore_identifiers.insert(segment.ident.to_string());
                                            }
                                        }
                                    }
                                },
                                _ => (),
                            }
                        }
                    }
                }
            }
        }

        Ok(IgnoreArgs {
            ignore_identifiers,
        })
    }
}

#[proc_macro_attribute]
pub fn world(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemStruct);
    let item_name = item.ident;

    let ignore_args = parse_macro_input!(attr as IgnoreArgs);

    let fields = item.fields;
    let mut field_identifiers = Vec::new();
    let mut field_types = Vec::new();
    let mut ignore_identifiers = Vec::new();
    let mut ignore_types = Vec::new();
    for field in fields.iter() {
        if let Some(ident) = &field.ident {
            if ignore_args.ignore_identifiers.contains(&ident.to_string()) {
                ignore_identifiers.push(ident);
                ignore_types.push(&field.ty);
            } else {
                field_identifiers.push(ident);
                field_types.push(&field.ty);
            }
        }
    }

    let plural_identifiers: Vec<Ident> = field_identifiers.iter().map(|v| format_ident!("{}s", v)).collect();
    let setter_identifiers: Vec<Ident> = field_identifiers.iter().map(|v| format_ident!("set_{}", v)).collect();
    let set_many_identifiers: Vec<Ident> = field_identifiers.iter().map(|v| format_ident!("set_{}s", v)).collect();
    let clear_identifiers: Vec<Ident> = field_identifiers.iter().map(|v| format_ident!("clear_{}", v)).collect();
    let clear_many_identifiers: Vec<Ident> = field_identifiers.iter().map(|v| format_ident!("clear{}s", v)).collect();
    let set_ignore_identifiers: Vec<Ident> = ignore_identifiers.iter().map(|v| format_ident!("set_{}", v)).collect();
    let clear_ignore_identifiers: Vec<Ident> = ignore_identifiers.iter().map(|v| format_ident!("clear_{}", v)).collect();

    let entity_fields = if field_identifiers.len() > 0 {
        quote!{
            #(pub #field_identifiers: std::sync::Arc<std::sync::RwLock<std::vec::Vec<std::option::Option<#field_types>>>>),*,
        }
    } else {
        quote!{ }
    };

    let entity_initializers = if field_identifiers.len() > 0 {
        quote!{
            #(#field_identifiers: std::sync::Arc::new(std::sync::RwLock::new(std::vec::Vec::new()))),*,
        }
    } else {
        quote!{ }
    };

    let global_fields = if ignore_identifiers.len() > 0 {
        quote!{
            #(pub #ignore_identifiers: std::sync::Arc<std::sync::RwLock<std::option::Option<#ignore_types>>>),*,
        }
    } else {
        quote!{ }
    };

    let global_initializers = if ignore_identifiers.len() > 0 {
        quote!{
            #(#ignore_identifiers: std::sync::Arc::new(std::sync::RwLock::new(None))),*,
        }
    } else {
        quote!{ }
    };

    TokenStream::from(quote!{
        pub struct #item_name {
            entities: std::sync::Arc<std::sync::RwLock<std::vec::Vec<usize>>>,
            #entity_fields
            #global_fields
        }

        impl #item_name {
            pub fn new() -> std::sync::Arc<std::sync::RwLock<Self>> {
                std::sync::Arc::new(std::sync::RwLock::new(Self {
                    entities: std::sync::Arc::new(std::sync::RwLock::new(std::vec::Vec::new())),
                    #entity_initializers
                    #global_initializers
                }))
            }

            pub fn add_entity(&mut self) -> usize {
                let entity_id = self.entities.read().unwrap().len() as usize;
                self.entities.write().unwrap().push(entity_id);
                #(self.#field_identifiers.write().unwrap().push(None));*;
                entity_id
            }

            pub fn add_entities(&mut self, entities: usize) -> Vec<usize> {
                let mut new_entity_ids = Vec::with_capacity(entities as usize);
                let mut entities_list = self.entities.write().unwrap();
                #(let mut #field_identifiers = self.#field_identifiers.write().unwrap());*;
                let start_len = entities_list.len();

                for i in 0..entities {
                    let new_entity_id = start_len + i;
                    entities_list.push(new_entity_id);
                     #(#field_identifiers.push(None));*;
                     new_entity_ids.push(new_entity_id);
                }

                new_entity_ids
            }

            pub fn remove_entity(&mut self, entity_id: usize) {
                self.entities.write().unwrap().remove(entity_id as usize);
                #(self.#field_identifiers.write().unwrap().remove(entity_id as usize));*;
            }

            pub fn remove_entities(&mut self, entity_ids: Vec<usize>) {
                for entity_id in entity_ids {
                    self.entities.write().unwrap().remove(entity_id as usize);
                    #(self.#field_identifiers.write().unwrap().remove(entity_id as usize));*;
                }
            }

            #(pub fn #setter_identifiers(&mut self, entity_id: usize, #field_identifiers: #field_types) {
                self.#field_identifiers.write().unwrap()[entity_id as usize] = Some(#field_identifiers);
            })*

            #(pub fn #set_ignore_identifiers(&mut self, #ignore_identifiers: #ignore_types) {
                *self.#ignore_identifiers.write().unwrap() = Some(#ignore_identifiers);
            })*

            #(pub fn #set_many_identifiers(&mut self, entity_ids: &Vec<usize>, mut #plural_identifiers: Vec<#field_types>) {
                let mut component = self.#field_identifiers.write().unwrap();
                for (#field_identifiers, entity_id) in #plural_identifiers.drain(..).zip(entity_ids.iter()) {
                    component[*entity_id] = Some(#field_identifiers);
                }
            })*

            #(pub fn #clear_identifiers(&mut self, entity_id: usize) {
                self.#field_identifiers.write().unwrap()[entity_id as usize] = None;
            })*

            #(pub fn #clear_ignore_identifiers(&mut self) {
                *self.#ignore_identifiers.write().unwrap() = None;
            })*

            #(pub fn #clear_many_identifiers(&mut self, entity_ids: &Vec<usize>) {
                let mut component = self.#field_identifiers.write().unwrap();
                for entity_id in entity_ids {
                    component[*entity_id as usize] = None;
                }
            })*
        }

        unsafe impl Send for #item_name {}
        unsafe impl Sync for #item_name {}
    })
}

struct WorldArgs {
    function_name: Ident,
    function_args: Vec<FnArg>,
    body: Block,
}

impl Parse for WorldArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let function_parts = ItemFn::parse(input)?;

        // Parsing System Name
        let function_name = function_parts.sig.ident;

        // Function Arguments
        let function_args: Vec<FnArg> = function_parts.sig.inputs.iter().map(|v| v.clone()).collect();

        Ok(WorldArgs {
            function_name,
            function_args,
            body: *function_parts.block,
        })
    }
}

struct FunctionArgs {
    world_type: Ident,
    read_components: Vec<Ident>,
    global_read_components: Vec<Ident>,
    write_components: Vec<Ident>,
    global_write_components: Vec<Ident>,
    global_write_assignments: HashMap<Ident, Expr>,
    filters: Vec<ExprBinary>,
    enumerated: bool,
}

impl Parse for FunctionArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut world_type: Option<Ident> = None;
        let mut read_components: Vec<Ident> = Vec::new();
        let mut global_read_components: Vec<Ident> = Vec::new();
        let mut write_components: Vec<Ident> = Vec::new();
        let mut global_write_components: Vec<Ident> = Vec::new();
        let mut global_write_assignments: HashMap<Ident, Expr> = HashMap::new();
        let mut filters: Vec<ExprBinary> = Vec::new();
        let mut enumerated: bool = false;

        let parts = input.parse_terminated(Expr::parse, Token![,])?;
        for part in parts.iter() {
            if let Expr::Assign(assignment) = part {
                match assignment.left.as_ref() {
                    Expr::Path(path) => {
                        if let Some(segment) = path.path.segments.first() {
                            match segment.ident.to_string().as_str() {
                                "world" => {
                                    if let Expr::Path(path) = assignment.right.as_ref() {
                                        if let Some(segment) = path.path.segments.first() {
                                            world_type = Some(segment.ident.clone());
                                        } else {
                                            return Err(Error::new(Span::call_site(), "Expected Singular Type for World Type"));
                                        }
                                    }
                                },
                                "read" => {
                                    match assignment.right.as_ref() {
                                        Expr::Path(path) => {
                                            if let Some(segment) = path.path.segments.first() {
                                                read_components.push(segment.ident.clone());
                                            }
                                        },
                                        Expr::Array(array) => {
                                            for element in array.elems.iter() {
                                                if let Expr::Path(path) = element {
                                                    if let Some(segment) = path.path.segments.first() {
                                                        read_components.push(segment.ident.clone());
                                                    }
                                                }
                                            }
                                        },
                                        _ => (),
                                    }
                                },
                                "write" => {
                                    match assignment.right.as_ref() {
                                        Expr::Path(path) => {
                                            if let Some(segment) = path.path.segments.first() {
                                                write_components.push(segment.ident.clone());
                                            }
                                        },
                                        Expr::Array(array) => {
                                            for element in array.elems.iter() {
                                                if let Expr::Path(path) = element {
                                                    if let Some(segment) = path.path.segments.first() {
                                                        write_components.push(segment.ident.clone());
                                                    }
                                                }
                                            }
                                        },
                                        _ => (),
                                    }
                                },
                                "filter" => {
                                    if let Expr::Array(array) = assignment.right.as_ref() {
                                        for element in array.elems.iter() {
                                            if let Expr::Binary(binary) = element {
                                                filters.push(binary.clone());
                                            }
                                        }
                                    }
                                },
                                "_read" => {
                                    match assignment.right.as_ref() {
                                        Expr::Path(path) => {
                                            if let Some(segment) = path.path.segments.first() {
                                                global_read_components.push(segment.ident.clone());
                                            }
                                        },
                                        Expr::Array(array) => {
                                            for element in array.elems.iter() {
                                                if let Expr::Path(path) = element {
                                                    if let Some(segment) = path.path.segments.first() {
                                                        global_read_components.push(segment.ident.clone());
                                                    }
                                                }
                                            }
                                        },
                                        _ => (),
                                    }
                                },
                                "_write" => {
                                    match assignment.right.as_ref() {
                                        Expr::Path(path) => {
                                            if let Some(segment) = path.path.segments.first() {
                                                global_write_components.push(segment.ident.clone());
                                            }
                                        },
                                        Expr::Array(array) => {
                                            for element in array.elems.iter() {
                                                match element {
                                                    Expr::Path(path) => {
                                                        if let Some(segment) = path.path.segments.first() {
                                                            global_write_components.push(segment.ident.clone());
                                                        }
                                                    },
                                                    Expr::Assign(assignment) => {
                                                        if let Expr::Path(path) = assignment.left.as_ref() {
                                                            if let Some(segment) = path.path.segments.first() {
                                                                global_write_components.push(segment.ident.clone());
                                                                global_write_assignments.insert(segment.ident.clone(), *assignment.right.clone());
                                                            }
                                                        }
                                                    },
                                                    _ => (),
                                                }
                                            }
                                        },
                                        _ => (),
                                    }
                                },
                                "enumerate" => enumerated = true,
                                _ => (),
                            }
                        }
                    },
                    _ => return Err(Error::new(Span::call_site(), "Invalid Parameter to system macro")),
                }
            } else {
                return Err(Error::new(Span::call_site(), "Expected Assignments in Attribute"));
            }
        }

        if world_type.is_none() {
            return Err(Error::new(Span::call_site(), "World Type was not Provided"));
        }

        Ok(FunctionArgs {
            world_type: world_type.unwrap(),
            read_components,
            global_read_components,
            write_components,
            global_write_components,
            global_write_assignments,
            filters,
            enumerated,
        })
    }
}

#[proc_macro_attribute]
pub fn system(attr: TokenStream, item: TokenStream) -> TokenStream {
    let world_args = parse_macro_input!(item as WorldArgs);
    let function_args = parse_macro_input!(attr as FunctionArgs);

    let fn_name = world_args.function_name;
    let fn_args = world_args.function_args;
    let body = world_args.body;

    let read_components = function_args.read_components;
    let global_read_components = function_args.global_read_components;
    let global_read_refs: Vec<Ident> = global_read_components.iter().map(|v| format_ident!("{}_ref", v)).collect();
    let write_components = function_args.write_components;
    let global_write_components = function_args.global_write_components;
    let global_write_refs: Vec<Ident> = global_write_components.iter().map(|v| format_ident!("{}_ref", v)).collect();
    let world_type = function_args.world_type;

    let (mut items, mut iterators) = match read_components.len() {
        0 => (quote!{ }, quote!{ }),
        1 => {
            let read_component = &read_components[0];
            (
                quote!{ (entity_id, #read_component) },
                quote!{#read_component.iter().enumerate()}
            )
        },
        _ => {
            let read_component1 = &read_components[0];
            let read_component2 = &read_components[1];
            (
                quote!{ ((entity_id, #read_component1), #read_component2) },
                quote!{ #read_component1.iter().enumerate().zip(#read_component2.iter())}
            )
        },
    };

    if read_components.len() > 2 {
        for read_component in read_components[2..].iter() {
            items = quote!{ (#items, #read_component) };
            iterators = quote!{#iterators.zip(#read_component.iter())};
        }
    }

    if read_components.len() == 0 {
        (items, iterators) = match write_components.len() {
            0 => (quote!{ }, quote!{ }),
            1 => {
                let write_component = &write_components[0];
                (
                    quote!{ (entity_id, #write_component) },
                    quote!{#write_component.iter_mut().enumerate()}
                )
            },
            _ => {
                let write_component1 = &write_components[0];
                let write_component2 = &write_components[1];
                (
                    quote!{ ((entity_id, #write_component1), #write_component2) },
                    quote!{#write_component1.iter_mut().enumerate().zip(#write_component2.iter_mut())}
                )
            },
        };
        if write_components.len() > 2 {
            for write_component in write_components[2..].iter() {
                items = quote!{ (#items, #write_component) };
                iterators = quote!{#iterators.zip(#write_component.iter_mut())};
            }
        }
    } else {
        for write_component in write_components.iter() {
            items = quote!{ (#items, #write_component) };
            iterators = quote!{#iterators.zip(#write_component.iter_mut())};
        }
    }

    let mut filter = quote!{ };
    let combined_length = read_components.len() + write_components.len();
    match combined_length {
        0 => (),
        1 => filter = quote!{ .filter(|v| v.1.is_some()) },
        2 => filter = quote!{ .filter(|v| v.0.1.is_some() && v.1.is_some()) },
        _ => {
            filter = quote!{ v.1.is_some() };
            for i in 1..(combined_length-1) {
                filter = quote!{ #filter && v };
                for _ in 0..i {
                    filter = quote!{ #filter.0 };
                }
                filter = quote!{ #filter.1.is_some() };
            }
            filter = quote!{ .filter(|v| #filter) };
        },
    }

    let mut filter_condition = quote!{ };
    if function_args.filters.len() > 0 {
        let first_filter = &function_args.filters[0];
        filter_condition = quote!{ if #first_filter };
        for filter in function_args.filters[1..].iter() {
            filter_condition = quote!{ #filter_condition && #filter };
        }
    }

    let body = if read_components.len() + write_components.len() > 0 {
        quote!{ 
            for #items in #iterators #filter {
                #(let #read_components = #read_components.as_ref().unwrap());*;
                #(let mut #write_components = #write_components.as_mut().unwrap());*;

                #filter_condition {
                    #body
                }
            }
        }
    } else if function_args.enumerated {
        quote!{ 
            for entity_id in 0..world.entities.read().unwrap().len() {
                #body
            }
        }
    } else {
        quote!{ #body }
    };

    let mut global_write_assignments = quote!{ };
    for key in function_args.global_write_assignments.keys() {
        let value = function_args.global_write_assignments.get(key).unwrap();
        global_write_assignments = quote!{
            #global_write_assignments
            *#key = #value;
        };
    }


    TokenStream::from(quote!{
        pub fn #fn_name(world: std::sync::Arc<std::sync::RwLock<#world_type>>, #(#fn_args),*) {
            let world = world.read().unwrap();
            #(let #read_components = world.#read_components.read().unwrap());*;
            #(let mut #write_components = world.#write_components.write().unwrap());*;
            #(let #global_read_refs = world.#global_read_components.read().unwrap());*;
            #(let #global_read_components = #global_read_refs.as_ref().expect("Global Components must not be None"));*;
            #(let mut #global_write_refs = world.#global_write_components.write().unwrap());*;
            #(let mut #global_write_components = #global_write_refs.as_mut().expect("Global Components must not be None"));*;
            #global_write_assignments

            #body
        }
    })
}