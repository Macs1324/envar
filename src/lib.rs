use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Field, Lit, Meta, PathArguments, Type};

/// # Envar
/// A derive macro to automatically parse environment variables into a struct.
/// The macro will look for environment variables with the same name as the struct fields.
/// If the environment variable is not found, the program will panic.
///
/// ## Example
/// ```rust
/// use envar::Envar;
/// #[derive(Envar)]
/// struct Config {
///    #[env = "DB_CONNECTION_PORT"]
///    port: u16,
///    #[env = "DB_CONNECTION_HOST"]
///    host: String,
///    debug: Option<bool>,
///}
/// fn main() {
///   let config = Config::new();
///   println!("Port: {}", config.port);
///   println!("Host: {}", config.host);
///   // If PORT and HOST are not found in the environment, the program will not compile.
///   // If DEBUG is not found, it will be None.
///   println!("Debug: {:?}", config.debug);
/// }
/// ```
/// The `env` attribute can be used to specify a different environment variable name.
/// If the attribute is not present, the environment variable name will be the same as the field name in uppercase.
/// ```rust
/// use envar::Envar;
/// #[derive(Envar)]
/// struct Config {
///   #[env = "DB_CONNECTION_PORT"]
///   port: u16,
///   host: String,
///}
/// ```
/// In this example, the environment variable for `port` will be `DB_CONNECTION_PORT` and the environment variable for `host` will be `HOST`.
///
///
#[proc_macro_derive(Envar, attributes(env))]
pub fn env_new(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let fields = match input.data {
        Data::Struct(data) => data.fields,
        _ => panic!("Envar is only supported on structs"),
    };

    let init_fields = fields.iter().map(|field| generate_field_init(field));

    let expanded = quote! {
        impl #name {
            pub fn new() -> Self {
                Self {
                    #(#init_fields)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}

fn generate_field_init(field: &Field) -> proc_macro2::TokenStream {
    let field_name = field.ident.as_ref().unwrap();
    let ty = &field.ty;
    let env_var_name = match find_env_attr(&field.attrs) {
        Some(name) => name,
        None => field_name.to_string().to_uppercase(),
    };

    let parse_logic = if is_option_type(&field.ty) {
        let inner_ty = extract_option_inner_type(&field.ty).unwrap();
        quote! {
            std::env::var(#env_var_name).ok().map(|val| val.parse::<#inner_ty>().expect("Failed to parse environment variable"))
        }
    } else {
        if std::env::var(&env_var_name).is_err() {
            panic!("Environment variable {} not found", env_var_name);
        }
        quote! {
            std::env::var(#env_var_name).expect(&format!("Environment variable {} not found", #env_var_name))
                .parse::<#ty>().expect(&format!("Failed to parse environment variable {}", #env_var_name))
        }
    };

    quote! {
        #field_name: #parse_logic,
    }
}
// Helper function to check if a field is of type Option<T> and extract T if it is.
fn extract_option_inner_type(ty: &Type) -> Option<proc_macro2::TokenStream> {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Option" {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(gen_arg) = args.args.first() {
                        return Some(quote! { #gen_arg });
                    }
                }
            }
        }
    }
    None
}
fn find_env_attr(attrs: &[Attribute]) -> Option<String> {
    // Simplified logic to extract the attribute that specifies the env var name
    for attr in attrs {
        if let Ok(Meta::NameValue(meta)) = attr.parse_meta() {
            if meta.path.is_ident("env") {
                if let Lit::Str(lit) = meta.lit {
                    return Some(lit.value());
                }
            }
        }
    }
    None
}

fn is_option_type(ty: &syn::Type) -> bool {
    // Simplified type check for Option<T>
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.first() {
            return segment.ident == "Option";
        }
    }
    false
}
