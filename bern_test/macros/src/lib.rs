extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::{Span, Ident};
use quote::{format_ident, quote};
use syn::{parse, spanned::Spanned, Attribute, Item, ItemFn, ItemMod, ReturnType, Type};
use syn::__private::fmt::format;


// adapted from https://github.com/knurling-rs/defmt/blob/main/firmware/defmt-test/macros/src/lib.rs

#[proc_macro_attribute]
pub fn tests(args: TokenStream, input: TokenStream) -> TokenStream {

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
    // todo: print error if config wrong
    /* parse user test module */
    let mut tests = vec![];
    let mut imports = vec![];
    let mut tear_down_code = vec![];
    for item in items {
        match item {
            Item::Fn(mut func) => {
                let mut test = false;
                let mut should_panic = false;
                let mut ignored = false;
                let mut set_up = false;
                let mut tear_down = false;

                let name = func.sig.ident.clone();
                for attr in func.attrs.iter() {
                    if attr.path.is_ident("test") {
                        test = true;
                    } else if attr.path.is_ident("should_panic") {
                        should_panic = true;
                    } else if attr.path.is_ident("ignored") {
                        ignored = true;
                    } else if attr.path.is_ident("tear_down") {
                        tear_down = true;
                    }
                }

                if test && !ignored {
                    tests.push(Test {
                        name,
                        func,
                        should_panic,
                    });
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
    let func_names = tests.iter().map(|t| &t.name);
    let func_blocks = tests.iter().map(|t| &t.func.block);
    let func_should_panic = tests.iter().map(|t| &t.should_panic);
    let calls = func_names.clone();
    let name_strings = tests.iter().map(|t| format!("{}", &t.name));
    let i = (0..calls.len()).map(syn::Index::from);
    let k = i.clone(); // meh
    let name_copy = name_strings.clone();
    let n_tests = tests.len() as u8;
    /* Create test module containing:
     * - a test runner
     * - the test function implementations
     */
    let mut tokens = quote! {
        mod #module_name {
            #(#imports)*

            // todo: fix
            use bern_test::{println, sprintln, print, sprint, term_green, term_red};
            use core::panic::PanicInfo;
            use core::sync::atomic::{AtomicBool, Ordering};

            static SHOULD_PANIC: AtomicBool = AtomicBool::new(false);

            pub fn runner() {
                if bern_test::autorun::is_enabled() {
                    let test_index = bern_test::autorun::get_next_test();
                    if test_index < #n_tests {
                        bern_test::autorun::set_next_test(test_index + 1);
                        run(test_index);
                    } else {
                        let successes = bern_test::autorun::get_success_count();
                        let summary =  match successes {
                            #n_tests => "ok",
                            _ => "FAILED",
                        };
                        println!(
                            "\ntest result: {}. {} passed; {} failed",
                            summary,
                            successes,
                            #n_tests - successes,
                        );
                         bern_test::autorun::disable();
                    }
                } else {
                    list_tests();
                    let test_index = match bern_test::console::handle_user_input() {
                        255 => {
                            bern_test::autorun::enable();
                            bern_test::autorun::set_next_test(1);
                            println!("\nrunning {} tests", #n_tests);
                            0
                        },
                        i => {
                            println!("");
                            i
                        },
                    };
                    run(test_index);
                    tear_down();
                }
            }

            fn list_tests() {
                println!("\n\n======== Bern Test v{} ========",
                    bern_test::get_version(),
                );
                #(
                    println!("[{}] {}::{}", #k, #module_name_string, #name_copy);
                )*
                println!("[255] run all tests");
                println!("Select test [0..{}]:", #n_tests-1);
            }

            fn run(index: u8) {
                match index {
                #(
                    #i => {
                        print!("test {}::{} ... ", #module_name_string, #name_strings);
                        /* setting boolean takes only one instruction */
                        SHOULD_PANIC.store(#func_should_panic, Ordering::Relaxed);
                        #calls();
                        /* if we get here the test did not panic */
                        if !#func_should_panic {
                            println!(term_green!("ok"));
                            bern_test::autorun::test_succeeded();
                        } else {
                            println!(term_red!("FAILED"));
                            //println!(" └─ did not panic");
                        }
                    },
                )*
                    _ => (),
                };
            }

            pub fn panicked(info: &PanicInfo) {
                if SHOULD_PANIC.load(Ordering::Relaxed) {
                    println!(term_green!("ok"));
                    bern_test::autorun::test_succeeded();
                } else {
                    println!(term_red!("FAILED"));
                    //println!(" └─ {}", info);
                }
                //println!(" └─ we're in the panic handler, waiting for reset ... ");
                tear_down();
            }

            fn tear_down() {
                #( #tear_down_code )*
            }

            #(
                fn #func_names() #func_blocks
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