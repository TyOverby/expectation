use proc_macro::TokenStream;
use quote::quote;
use syn::ItemFn;

#[proc_macro_attribute]
pub fn expectation_test(_metadata: TokenStream, input: TokenStream) -> TokenStream {
    let item: ItemFn = syn::parse(input).expect("failed to parse input");
    let old_name = &item.sig.ident;
    let new_name_str = format!("expectation_test_{}", old_name);
    let new_name = syn::Ident::new(&new_name_str, old_name.span());
    let old_name_lit = syn::LitStr::new(&new_name_str, old_name.span());

    let output = quote! {
        #[test]
        fn #new_name () {
            #item
            ::expectation::expect(
                #old_name_lit,
                #old_name,
            );
        }
    };
    output.into()
}
