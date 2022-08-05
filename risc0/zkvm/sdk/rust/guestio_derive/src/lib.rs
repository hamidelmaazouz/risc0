use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Serialize)]
pub fn derive_serialize(input: proc_macro::TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let fields = match input.data {
        syn::Data::Struct(st) => st.fields,
        _ => panic!("Not a struct! {:#?}", input.data),
    };

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let field_names: Vec<_> = fields.iter().map(|f| f.ident.as_ref().unwrap()).collect();

    let field_ty: Vec<_> = fields.iter().map(|f| &f.ty).collect();

    let name = input.ident;
    let derive_tokens = quote! {
        impl #impl_generics guestio::Serialize for #name #ty_generics #where_clause {
            const FIXED_WORDS : usize =
                #(<#field_ty as guestio::Serialize>:: FIXED_WORDS)+*;

            fn tot_len(&self) -> usize {
                #(self . #field_names . tot_len()
                )+*
            }

            fn fill(&self, buf: &mut guestio::AllocBuf, a: &mut guestio::Alloc) -> guestio::Result<()> {
                let pos: usize = 0;
                #(
                    self. #field_names . fill(&mut buf.descend(pos), a)?;
                    let pos = pos + <#field_ty as guestio::Serialize>::fixed_len();
                );*

                Ok(())
            }
        }
    };

    eprintln!("{}", derive_tokens);

    derive_tokens.into()
}

#[proc_macro_derive(Deserialize)]
pub fn derive_deserialize(input: proc_macro::TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let fields = match input.data {
        syn::Data::Struct(st) => st.fields,
        _ => panic!("Not a struct! {:#?}", input.data),
    };

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let field_names: Vec<_> = fields.iter().map(|f| f.ident.as_ref().unwrap()).collect();

    let field_ty: Vec<_> = fields.iter().map(|f| &f.ty).collect();

    let field_offsets: Vec<_> = (0..field_names.len())
        .map(|idx| {
            if idx == 0 {
                quote! { 0 }
            } else {
                let part_ty = &field_ty[0..idx];
                quote! {
                    #( <#part_ty as guestio::Deserialize<'_>> :: FIXED_WORDS)+*
                }
            }
        })
        .collect();

    let name = input.ident;
    let ref_name = format_ident!("{}Ref", name);

    let derive_tokens = quote! {
            struct #ref_name <'a> {
                buf: &'a [u32],
            }

            impl<'a> #ref_name <'a> {
                #(
                    fn #field_names(&self) -> <#field_ty as guestio::Deserialize<'a>>::RefType {
                        <#field_ty as guestio::Deserialize<'a>>::deserialize_from(
                            &self.buf[#field_offsets ..]
                        )
                    }
                )*
            }

            impl <'a> #impl_generics guestio::Deserialize<'a> for #name #ty_generics #where_clause {
                type RefType = #ref_name <'a>;
                type OrigType = #name;
                const FIXED_WORDS : usize =
                    #(<#field_ty as guestio::Deserialize<'_>>:: FIXED_WORDS)+* ;

                fn deserialize_from(buf: &'a [u32]) -> Self::RefType {
                    #ref_name { buf }
                }

                fn into_orig(val: Self::RefType) -> Self::OrigType{val.into()}
            }

            impl<'a> std::convert::From< #ref_name <'a> > for #name {
                fn from(val: #ref_name <'a>) -> Self {
                    #name {
                        #(
                            #field_names :
                            <#field_ty as guestio::Deserialize<'_>>::into_orig(val.#field_names ()),
                        )*
                    }
                }
            }
        };

    eprintln!("{}", derive_tokens);

    derive_tokens.into()
}
