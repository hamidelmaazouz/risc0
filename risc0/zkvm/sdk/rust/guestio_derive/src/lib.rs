use proc_macro::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, punctuated::Punctuated, token::Comma, DeriveInput, GenericParam, Ident,
    Lifetime, LifetimeDef,
};

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
                    self. #field_names . fill(&mut buf.descend(pos, <#field_ty as guestio::Serialize>::FIXED_WORDS)?, a)?;
                    let pos = pos + <#field_ty as guestio::Serialize>::FIXED_WORDS;
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

    let (_impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let mut ref_generics = input.generics.clone();
    ref_generics
        .params
        .push(GenericParam::Lifetime(LifetimeDef::new(Lifetime::new(
            "'guestio_deserialize",
            Span::call_site().into(),
        ))));
    let (ref_impl_generics, ref_ty_generics, ref_where_clause) = ref_generics.split_for_impl();
    let ref_ty_generics_turbofish = ref_ty_generics.as_turbofish();

    let phantom_types = Punctuated::<Ident, Comma>::from_iter(
        input.generics.params.iter().filter_map(|p| match p {
            GenericParam::Type(p) => Some(p.ident.clone()),
            _ => None,
        }),
    );

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
            struct #ref_name #ref_ty_generics {
                buf: &'guestio_deserialize [u32],
                phantom: core::marker::PhantomData <(#phantom_types)>,
            }

            impl #ref_impl_generics #ref_name #ref_ty_generics #ref_where_clause {
                #(
                    fn #field_names(&self) -> <#field_ty as guestio::Deserialize<'guestio_deserialize>>::RefType {
                        <#field_ty as guestio::Deserialize<'guestio_deserialize>>::deserialize_from(
                            &self.buf[#field_offsets ..]
                        )
                    }
                )*
            }

           impl #ref_impl_generics guestio::Deserialize<'guestio_deserialize> for #name #ty_generics #where_clause {
                type RefType = #ref_name #ref_ty_generics;
                const FIXED_WORDS : usize =
                    #(<#field_ty as guestio::Deserialize<'_>>:: FIXED_WORDS)+* ;

                fn deserialize_from(buf: &'guestio_deserialize [u32]) -> Self::RefType {
                    #ref_name #ref_ty_generics_turbofish { buf, phantom: core::marker::PhantomData }
                }

                fn into_orig(val: Self::RefType) -> Self{val.into()}
           }

            impl #ref_impl_generics std::convert::From< #ref_name #ref_ty_generics > for #name #ty_generics #ref_where_clause {
                fn from(val: #ref_name #ref_ty_generics) -> Self {
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
