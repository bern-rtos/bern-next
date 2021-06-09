extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{parse, spanned::Spanned, Item, ItemFn, ItemMod, FnArg, Pat};


// adapted from https://github.com/knurling-rs/defmt/blob/main/firmware/defmt-test/macros/src/lib.rs

#[proc_macro_attribute]
pub fn tests(_args: TokenStream, input: TokenStream) -> TokenStream {
    let module: ItemMod = syn::parse(input).unwrap();

    let items = if let Some(content) = module.content {
        content.1
    } else {
        return parse::Error::new(
            module.span(),
            "module must be inline (e.g. `mod foo {}`)",
        ).to_compile_error().into();
    };

    // todo: make parser clean and extensible
    // todo: print error if config is invalid
    /* parse user test module */
    let mut tests = vec![];
    let mut imports = vec![];
    let mut test_set_up_code = vec![];
    let mut test_tear_down_code = vec![];
    let mut tear_down_code = vec![];
    let mut test_input_idents = vec![];
    let mut test_input_types = vec![];
    for item in items {
        match item {
            Item::Fn(func) => {
                let mut test = false;
                let mut should_panic = false;
                let mut ignored = false;
                let mut test_set_up = false;
                let mut test_tear_down = false;
                let mut tear_down = false;

                let name = func.sig.ident.clone();
                for attr in func.attrs.iter() {
                    if attr.path.is_ident("test") {
                        test = true;
                    } else if attr.path.is_ident("should_panic") {
                        should_panic = true;
                    } else if attr.path.is_ident("ignore") {
                        ignored = true;
                    } else if attr.path.is_ident("test_set_up") {
                        test_set_up = true;
                    } else if attr.path.is_ident("test_tear_down") {
                        test_tear_down = true;
                    } else if attr.path.is_ident("tear_down") {
                        tear_down = true;
                    }
                }

                /* parse test input parameter list */
                if test {
                    let mut idents = vec![];
                    let mut types = vec![];
                    for arg in func.sig.inputs.iter() {
                        if let FnArg::Typed(pat) = arg {
                            if let Pat::Ident(patid) = *pat.pat.clone() {
                                idents.push(patid);
                                types.push(*pat.ty.clone());
                            }
                        } else {
                            // self not supported
                        }
                    }
                    if test_input_types.len() == 0 {
                        test_input_types = types;
                        test_input_idents = idents;
                    } else {
                        // todo: check params
                    }
                }

                if test && !ignored {
                    tests.push(Test {
                        name,
                        func,
                        should_panic,
                    });
                } else if test_set_up {
                    test_set_up_code = func.block.stmts;
                } else if test_tear_down {
                    test_tear_down_code = func.block.stmts;
                } else if tear_down {
                    tear_down_code = func.block.stmts;
                }
            }

            Item::Use(u) => {
                imports.push(u);
            }

            _ => {
                return parse::Error::new(
                    item.span(),
                    "only `#[test]` functions and imports (`use`) are allowed in this scope",
                ).to_compile_error().into();
            }
        }
    }

    // todo: clean
    let module_name = module.ident.clone();
    let module_name_string = format!("{}", module.ident);
    let test_blocks = tests.iter().map(|t| &t.func.block);
    let test_should_panic = tests.iter().map(|t| &t.should_panic);
    let test_sig = tests.iter().map(|t| &t.func.sig);


    let test_input_declaration = quote! {
        #(#test_input_idents: #test_input_types,)*
    };
    let test_input_names = test_input_idents.iter().map(|i| {
        &i.ident
    });
    let test_input_call = quote! {
        #(#test_input_names,)*
    };
    let test_calls = tests.iter().map(|t| {
        let call = &t.name;
        match t.func.sig.inputs.len() {
            0 => quote! { #call(); },
            _ => quote! { #call(#test_input_call); },
        }
    });

    let name_strings = tests.iter().map(|t| format!("{}", &t.name));
    let i = (0..test_calls.len()).map(syn::Index::from);
    let k = i.clone(); // meh
    let name_copy = name_strings.clone();
    let n_tests = tests.len() as u8;
    /* Create test module containing:
     * - a test runner
     * - the test function implementations
     */
    let tokens = quote! {
        mod #module_name {
            #(#imports)*

            use bern_test::{println, print, term_green, term_red, term_reset, term_gray};
            use core::panic::PanicInfo;
            use core::sync::atomic::{AtomicBool, Ordering};

            static SHOULD_PANIC: AtomicBool = AtomicBool::new(false);

            pub fn runner(#test_input_declaration) {
                if bern_test::is_autorun_enabled() && !bern_test::run_all::is_active() {
                    __print_header();
                    __runall_initiate();
                } else if !bern_test::run_all::is_active() {
                    // provide user interface
                    __print_header();
                    __list_tests();
                    let test_index = match bern_test::console::handle_user_input() {
                        255 => {
                            __runall_initiate();
                        },
                        i => {
                            println!("");
                            __test_set_up();
                            __run(i, #test_input_call);
                            __test_tear_down();
                        },
                    };
                }

                if bern_test::run_all::is_active() {
                    __runall(#test_input_call);
                }
            }

            fn __print_header() {
                println!(term_reset!());
                println!("~~~~~~~~~~~~~~ Bern Test v{} ~~~~~~~~~~~~~~",
                    bern_test::get_version(),
                );
            }

            fn __list_tests() {
                #(
                    println!("[{}] {}::{}", #k, #module_name_string, #name_copy);
                )*
                println!("[255] run all tests");
                println!("Select test [0..{}]:", #n_tests-1);
            }

            fn __runall_initiate() {
                bern_test::run_all::activate();
                bern_test::run_all::set_next_test(0);
                println!("\nrunning {} tests", #n_tests);
            }

            fn __runall(#test_input_declaration) {
                let test_index = bern_test::run_all::get_next_test();
                if test_index < #n_tests {
                    bern_test::run_all::set_next_test(test_index + 1);
                    __test_set_up();
                    __run(test_index, #test_input_call);
                    __test_tear_down();
                } else {
                    let successes = bern_test::run_all::get_success_count();
                    let summary =  match successes {
                        #n_tests => term_green!("ok"),
                        _ => term_red!("FAILED"),
                    };
                    println!(
                        "\ntest result: {}. {} passed; {} failed",
                        summary,
                        successes,
                        #n_tests - successes,
                    );
                    bern_test::run_all::deactivate();
                    __tear_down();
                }
            }

            fn __run(index: u8, #test_input_declaration) {
                match index {
                #(
                    #i => {
                        print!("test {}::{} ... ", #module_name_string, #name_strings);
                        /* setting boolean takes only one instruction */
                        SHOULD_PANIC.store(#test_should_panic, Ordering::SeqCst);
                        #test_calls
                        /* if we get here the test did not panic */
                        if !#test_should_panic {
                            bern_test::test_succeeded();
                        } else {
                            bern_test::test_failed(" └─ did not panic");
                        }
                    },
                )*
                    _ => (),
                };
            }

            pub fn panicked(info: &PanicInfo) {
                if SHOULD_PANIC.load(Ordering::Relaxed) {
                    bern_test::test_succeeded();
                } else {
                    bern_test::test_panicked(info);
                }
                __test_tear_down();
            }

            // runs before every test
            fn __test_set_up() {
                #( #test_set_up_code )*
            }

            // runs after every test
            fn __test_tear_down() {
                #( #test_tear_down_code )*
            }

            // runs after all tests
            fn __tear_down() {
                #( #tear_down_code )*
            }

            #(
                #test_sig #test_blocks
            )*
        }

        use core::panic::PanicInfo;
        use core::sync::atomic::{self, Ordering};

        #[panic_handler]
        fn panic(info: &PanicInfo) -> ! {
            #module_name::panicked(info);
            loop {
                atomic::compiler_fence(Ordering::SeqCst);
            }
        }
    };
    return TokenStream::from(tokens);
}


struct Test {
    name: Ident,
    func: ItemFn,
    should_panic: bool,
}