extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote, ToTokens};
use syn::{parse::{Parse, ParseStream, Result}, parse_macro_input, Token, Block, Error, Expr, FnArg, Ident, ItemFn, ItemStruct, Pat, PatType};

#[proc_macro_attribute]
pub fn world(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemStruct);
    let item_name = item.ident;

    let fields = item.fields;
    let mut field_identifiers = Vec::new();
    let mut field_types = Vec::new();
    for field in fields.iter() {
        if let Some(ident) = &field.ident {
            field_identifiers.push(ident);
            field_types.push(&field.ty);
        }
    }

    let setter_identifiers: Vec<Ident> = field_identifiers.iter().map(|v| format_ident!("set_{}", v)).collect();
    let clear_identifiers: Vec<Ident> = field_identifiers.iter().map(|v| format_ident!("clear_{}", v)).collect();

    TokenStream::from(quote!{
        pub struct #item_name {
            entities: std::sync::Arc<std::sync::RwLock<std::vec::Vec<u32>>>,
            #(pub #field_identifiers: std::sync::Arc<std::sync::RwLock<std::vec::Vec<std::option::Option<#field_types>>>>),*
        }

        impl #item_name {
            pub fn new() -> Self {
                Self {
                    entities: std::sync::Arc::new(std::sync::RwLock::new(std::vec::Vec::new())),
                    #(#field_identifiers: std::sync::Arc::new(std::sync::RwLock::new(std::vec::Vec::new()))),*
                }
            }

            pub fn add_entity(&mut self) -> u32 {
                let entity_id = self.entities.read().unwrap().len() as u32;
                self.entities.write().unwrap().push(entity_id);
                #(self.#field_identifiers.write().unwrap().push(None));*;
                entity_id
            }

            #(pub fn #setter_identifiers(&mut self, entity_id: u32, #field_identifiers: #field_types) {
                self.#field_identifiers.write().unwrap()[entity_id as usize] = Some(#field_identifiers);
            })*

            #(pub fn #clear_identifiers(&mut self, entity_id: u32) {
                self.#field_identifiers.write().unwrap()[entity_id as usize] = None;
            })*
        }
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
    write_components: Vec<Ident>,
}

impl Parse for FunctionArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut world_type: Option<Ident> = None;
        let mut read_components: Vec<Ident> = Vec::new();
        let mut write_components: Vec<Ident> = Vec::new();

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
                                _ => (),
                            }
                        }
                    },
                    _ => println!("Nope"),
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
            write_components,
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
    let write_components = function_args.write_components;
    let world_type = function_args.world_type;

    let (mut items, mut iterators) = match read_components.len() {
        0 => (quote!{ }, quote!{ }),
        1 => {
            let read_component = &read_components[0];
            (
                quote!{ #read_component },
                quote!{#read_component.iter()}
            )
        },
        _ => {
            let read_component1 = &read_components[0];
            let read_component2 = &read_components[1];
            (
                quote!{ (#read_component1, #read_component2) },
                quote!{ #read_component1.iter().zip(#read_component2.iter())}
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
                    quote!{ #write_component },
                    quote!{#write_component.iter_mut()}
                )
            },
            _ => {
                let write_component1 = &write_components[0];
                let write_component2 = &write_components[1];
                (
                    quote!{ (#write_component1, #write_component2) },
                    quote!{#write_component1.iter_mut()().zip(#write_component2.iter_mut())}
                )
            },
        };
        if write_components.len() > 2 {
            for write_component in write_components.iter() {
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

    // TODO: Write Filter Part Macro

    TokenStream::from(quote!{
        pub fn #fn_name(world: std::sync::Arc<#world_type>, #(#fn_args),*) {
            #(let #read_components = world.#read_components.read().unwrap());*;
            #(let mut #write_components = world.#write_components.write().unwrap());*;

            for #items in #iterators {
                #body
            }
        }
    })
}