#![recursion_limit = "256"]

extern crate proc_macro;
use proc_macro::*;
use quote::quote;
use syn::*;

#[proc_macro_attribute]
pub fn field(attr: TokenStream, item: TokenStream) -> proc_macro::TokenStream {
    let item_ast: DeriveInput = parse(item.clone()).unwrap(); // TODO: make safe
    let struct_name = &item_ast.ident;
    let mod_str = attr.into_iter().next().unwrap().to_string(); // TODO: make safe
    let mod_str_len = mod_str.len();
    let bits = mod_str_len * 8;
    let bytes = (bits + 7) / 8;

    let new_item = quote! {
        #[derive(Clone, Copy)]
        struct #struct_name {
            b: [u8; #bytes]
        }

        impl #struct_name {
            pub fn new() -> Self {
                Self {
                    b: [0u8; #bytes],
                }
            }

            fn max() -> BigUint {
                BigUint::from(2u32).shl(#bits)
            }

            fn mod_val(&self) -> BigUint {
                // TODO: make safe
                BigUint::from_str_radix(#mod_str, 16).unwrap()
            }

            #[allow(dead_code)]
            pub fn from_literal(x: u128) -> Self {
                let big_x = BigUint::from(x);
                // TODO: wrap here?
                if big_x > #struct_name::max().into() {
                    panic!("literal too big for type {}", stringify!(#struct_name));
                }
                big_x.into()
            }

            #[allow(dead_code)]
            pub fn from_hex(x: &str) -> Self {
                let big_x = BigUint::from_str_radix(x, 16)
                    .unwrap_or_else(|_| panic!("string is not a valid hex number {}", x));
                if big_x > #struct_name::max().into() {
                    panic!("literal too big for type {}", stringify!(#struct_name));
                }
                big_x.into()
            }

            /// Returns 2 to the power of the argument
            #[allow(dead_code)]
            pub fn pow2(x: usize) -> #struct_name {
                BigUint::from(1u32).shl(x).into()
            }

            #[allow(dead_code)]
            pub fn to_bytes_le(&self) -> Vec<u8> {
                BigUint::from_bytes_be(&self.b).to_bytes_le()
            }
        }

        impl From<BigUint> for #struct_name {
            fn from(x: BigUint) -> #struct_name {
                let repr = x.to_bytes_be();
                if repr.len() > #bytes {
                    panic!("BigUint too big for type {}", stringify!(#struct_name))
                }
                let mut out = [0u8; #bytes];
                let upper = out.len();
                let lower = upper - repr.len();
                out[lower..upper].copy_from_slice(&repr);
                #struct_name{b: out}
            }
        }

        impl Into<BigUint> for #struct_name {
            fn into(self) -> BigUint {
                BigUint::from_bytes_be(&self.b)
            }
        }

        impl std::fmt::Display for #struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                let uint: BigUint = (*self).into();
                write!(f, "{}", uint)
            }
        }

        impl std::fmt::Debug for #struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                let uint: BigUint = (*self).into();
                write!(f, "{}", uint)
            }
        }

        impl std::cmp::PartialEq for #struct_name {
            fn eq(&self, rhs: &#struct_name) -> bool {
                let a: BigUint = (*self).into();
                let b: BigUint = (*rhs).into();
                a == b
            }
        }

        impl Eq for #struct_name {}

        impl PartialOrd for #struct_name {
            fn partial_cmp(&self, other: &#struct_name) -> Option<std::cmp::Ordering> {
                let a: BigUint = (*self).into();
                let b: BigUint = (*other).into();
                a.partial_cmp(&b)
            }
        }

        impl Ord for #struct_name {
            fn cmp(&self, other: &#struct_name) -> std::cmp::Ordering {
                self.partial_cmp(other).unwrap()
            }
        }

        /// **Warning**: wraps on overflow.
        impl Add for #struct_name {
            type Output = #struct_name;
            fn add(self, rhs: #struct_name) -> #struct_name {
                let a: BigUint = self.into();
                let b: BigUint = rhs.into();
                let c: BigUint = a + b;
                let d: BigUint = c % self.mod_val();
                d.into()
            }
        }

        /// **Warning**: wraps on underflow.
        impl Sub for #struct_name {
            type Output = #struct_name;
            fn sub(self, rhs: #struct_name) -> #struct_name {
                let a: BigUint = self.into();
                let b: BigUint = rhs.into();
                let c: BigUint = if b > a { self.mod_val() - b + a } else { b - a };
                c.into()
            }
        }

        /// **Warning**: wraps on overflow.
        impl Mul for #struct_name {
            type Output = #struct_name;
            fn mul(self, rhs: #struct_name) -> #struct_name {
                let a: BigUint = self.into();
                let b: BigUint = rhs.into();
                let c: BigUint = a * b;
                let d: BigUint = c % self.mod_val();
                d.into()
            }
        }

        /// **Warning**: panics on division by 0.
        impl Div for #struct_name {
            type Output = #struct_name;
            fn div(self, rhs: #struct_name) -> #struct_name {
                let a: BigUint = self.into();
                let b: BigUint = rhs.into();
                let c: BigUint = a / b;
                c.into()
            }
        }

        /// **Warning**: panics on division by 0.
        impl Rem for #struct_name {
            type Output = #struct_name;
            fn rem(self, rhs: #struct_name) -> #struct_name {
                let a: BigUint = self.into();
                let b: BigUint = rhs.into();
                let c: BigUint = a % b;
                c.into()
            }
        }
    };

    let new_item = TokenStream::from(new_item);
    new_item
}
