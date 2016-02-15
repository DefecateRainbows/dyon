use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use rand::Rng;

use runtime::{Expect, Flow, Runtime, Side};
use ast;

use Variable;
use Module;

pub fn standard() -> HashMap<&'static str, Intrinsic> {
    let mut i: HashMap<&'static str, Intrinsic> = HashMap::new();
    i.insert("println", PRINTLN);
    i.insert("print", PRINT);
    i.insert("clone", CLONE);
    i.insert("debug", DEBUG);
    i.insert("backtrace", BACKTRACE);
    i.insert("sleep", SLEEP);
    i.insert("round", ROUND);
    i.insert("random", RANDOM);
    i.insert("read_number", READ_NUMBER);
    i.insert("read_line", READ_LINE);
    i.insert("len", LEN);
    i.insert("push", PUSH);
    i.insert("trim_right", TRIM_RIGHT);
    i.insert("to_string", TO_STRING);
    i.insert("typeof", TYPEOF);
    i.insert("sqrt", SQRT);
    i.insert("sin", SIN);
    i.insert("asin", ASIN);
    i.insert("cos", COS);
    i.insert("acos", ACOS);
    i.insert("tan", TAN);
    i.insert("atan", ATAN);
    i.insert("exp", EXP);
    i.insert("ln", LN);
    i.insert("log2", LOG2);
    i.insert("log10", LOG10);
    i.insert("random", RANDOM);
    i.insert("load", LOAD);
    i.insert("load_source_imports", LOAD_SOURCE_IMPORTS);
    i.insert("call", CALL);
    i
}

fn deep_clone(v: &Variable, stack: &Vec<Variable>) -> Variable {
    use Variable::*;

    match *v {
        F64(_) => v.clone(),
        Return => v.clone(),
        Bool(_) => v.clone(),
        Text(_) => v.clone(),
        Object(ref obj) => {
            let mut res = obj.clone();
            for (_, val) in &mut res {
                *val = deep_clone(val, stack);
            }
            Object(res)
        }
        Array(ref arr) => {
            let mut res = arr.clone();
            for it in &mut res {
                *it = deep_clone(it, stack);
            }
            Array(res)
        }
        Ref(ind) => {
            deep_clone(&stack[ind], stack)
        }
        UnsafeRef(_) => panic!("Unsafe reference can not be cloned"),
        RustObject(_) => v.clone()
    }
}

fn print_variable(rt: &Runtime, v: &Variable) {
    match *rt.resolve(v) {
        Variable::Text(ref t) => {
            print!("{}", t);
        }
        Variable::F64(x) => {
            print!("{}", x);
        }
        Variable::Bool(x) => {
            print!("{}", x);
        }
        Variable::Ref(ind) => {
            print_variable(rt, &rt.stack[ind]);
        }
        Variable::Object(ref obj) => {
            print!("{{");
            let n = obj.len();
            for (i, (k, v)) in obj.iter().enumerate() {
                print!("{}: ", k);
                print_variable(rt, v);
                if i + 1 < n {
                    print!(", ");
                }
            }
            print!("}}");
        }
        Variable::Array(ref arr) => {
            print!("[");
            let n = arr.len();
            for (i, v) in arr.iter().enumerate() {
                print_variable(rt, v);
                if i + 1 < n {
                    print!(", ");
                }
            }
            print!("]");
        }
        ref x => panic!("Could not print out `{:?}`", x)
    }
}

pub fn call_standard(
    rt: &mut Runtime,
    call: &ast::Call,
    module: &Module
) -> (Expect, Flow) {
    let st = rt.stack.len();
    let lc = rt.local_stack.len();
    for arg in &call.args {
        match rt.expression(arg, Side::Right, module) {
            (x, Flow::Return) => { return (x, Flow::Return); }
            (Expect::Something, Flow::Continue) => {}
            _ => panic!("Expected something from argument")
        };
    }
    let expect = match &**call.name {
        "clone" => {
            rt.push_fn(call.name.clone(), st + 1, lc);
            let v = rt.stack.pop()
                .expect("There is no value on the stack");
            let v = deep_clone(rt.resolve(&v), &rt.stack);
            rt.stack.push(v);
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "println" => {
            rt.push_fn(call.name.clone(), st, lc);
            let x = rt.stack.pop()
                .expect("There is no value on the stack");
            print_variable(rt, &x);
            println!("");
            rt.pop_fn(call.name.clone());
            Expect::Nothing
        }
        "print" => {
            rt.push_fn(call.name.clone(), st, lc);
            let x = rt.stack.pop()
                .expect("There is no value on the stack");
            print_variable(rt, &x);
            rt.pop_fn(call.name.clone());
            Expect::Nothing
        }
        "sqrt" => rt.unary_f64(|a| a.sqrt()),
        "sin" => rt.unary_f64(|a| a.sin()),
        "asin" => rt.unary_f64(|a| a.asin()),
        "cos" => rt.unary_f64(|a| a.cos()),
        "acos" => rt.unary_f64(|a| a.acos()),
        "tan" => rt.unary_f64(|a| a.tan()),
        "atan" => rt.unary_f64(|a| a.atan()),
        "exp" => rt.unary_f64(|a| a.exp()),
        "ln" => rt.unary_f64(|a| a.ln()),
        "log2" => rt.unary_f64(|a| a.log2()),
        "log10" => rt.unary_f64(|a| a.log10()),
        "sleep" => {
            use std::thread::sleep;
            use std::time::Duration;

            rt.push_fn(call.name.clone(), st, lc);
            let v = match rt.stack.pop() {
                Some(Variable::F64(b)) => b,
                Some(_) => panic!("Expected number"),
                None => panic!("There is no value on the stack")
            };
            let secs = v as u64;
            let nanos = (v.fract() * 1.0e9) as u32;
            sleep(Duration::new(secs, nanos));
            rt.pop_fn(call.name.clone());
            Expect::Nothing
        }
        "random" => {
            rt.push_fn(call.name.clone(), st + 1, lc);
            let v = Variable::F64(rt.rng.gen());
            rt.stack.push(v);
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "round" => {
            rt.push_fn(call.name.clone(), st + 1, lc);
            let v = match rt.stack.pop() {
                Some(Variable::F64(b)) => b,
                Some(_) => panic!("Expected number"),
                None => panic!("There is no value on the stack")
            };
            let v = Variable::F64(v.round());
            rt.stack.push(v);
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "len" => {
            rt.push_fn(call.name.clone(), st + 1, lc);
            let v = match rt.stack.pop() {
                Some(v) => v,
                None => panic!("There is no value on the stack")
            };

            let v = {
                let arr = match rt.resolve(&v) {
                    &Variable::Array(ref arr) => arr,
                    _ => panic!("Expected array")
                };
                Variable::F64(arr.len() as f64)
            };
            rt.stack.push(v);
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "push" => {
            rt.push_fn(call.name.clone(), st + 1, lc);
            let item = match rt.stack.pop() {
                Some(item) => item,
                None => panic!("There is no value on the stack")
            };
            let v = match rt.stack.pop() {
                Some(v) => v,
                None => panic!("There is no value on the stack")
            };

            if let Variable::Ref(ind) = v {
                if let Variable::Array(ref mut arr) =
                rt.stack[ind] {
                    arr.push(item);
                } else {
                    panic!("Expected reference to array");
                }
            } else {
                panic!("Expected reference to array");
            }
            rt.pop_fn(call.name.clone());
            Expect::Nothing
        }
        "read_line" => {
            use std::io::{self, Write};

            rt.push_fn(call.name.clone(), st + 1, lc);
            let mut input = String::new();
            io::stdout().flush().unwrap();
            match io::stdin().read_line(&mut input) {
                Ok(_) => {}
                Err(error) => panic!("{}", error)
            };
            rt.stack.push(Variable::Text(Arc::new(input)));
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "read_number" => {
            use std::io::{self, Write};

            rt.push_fn(call.name.clone(), st + 1, lc);
            let err = match rt.stack.pop() {
                Some(Variable::Text(t)) => t,
                Some(_) => panic!("Expected text"),
                None => panic!("There is no value on the stack")
            };
            let stdin = io::stdin();
            let mut stdout = io::stdout();
            let mut input = String::new();
            loop {
                stdout.flush().unwrap();
                match stdin.read_line(&mut input) {
                    Ok(_) => {}
                    Err(error) => panic!("{}", error)
                };
                match input.trim().parse::<f64>() {
                    Ok(v) => {
                        rt.stack.push(Variable::F64(v));
                        break;
                    }
                    Err(_) => {
                        println!("{}", err);
                    }
                }
            }
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "trim_right" => {
            rt.push_fn(call.name.clone(), st + 1, lc);
            let mut v = match rt.stack.pop() {
                Some(Variable::Text(t)) => t,
                Some(_) => panic!("Expected text"),
                None => panic!("There is no value on the stack")
            };
            {
                let w = Arc::make_mut(&mut v);
                while let Some(ch) = w.pop() {
                    if !ch.is_whitespace() { w.push(ch); break; }
                }
            }
            rt.stack.push(Variable::Text(v));
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "to_string" => {
            rt.push_fn(call.name.clone(), st + 1, lc);
            let v = match rt.stack.pop() {
                Some(v) => v,
                None => panic!("There is no value on the stack")
            };
            let v = match rt.resolve(&v) {
                &Variable::Text(ref t) => Variable::Text(t.clone()),
                &Variable::F64(v) => {
                    Variable::Text(Arc::new(format!("{}", v)))
                }
                _ => unimplemented!(),
            };
            rt.stack.push(v);
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "typeof" => {
            rt.push_fn(call.name.clone(), st + 1, lc);
            let v = match rt.stack.pop() {
                Some(v) => v,
                None => panic!("There is no value on the stack")
            };
            let v = match rt.resolve(&v) {
                &Variable::Text(_) => rt.text_type.clone(),
                &Variable::F64(_) => rt.f64_type.clone(),
                &Variable::Return => rt.return_type.clone(),
                &Variable::Bool(_) => rt.bool_type.clone(),
                &Variable::Object(_) => rt.object_type.clone(),
                &Variable::Array(_) => rt.array_type.clone(),
                &Variable::Ref(_) => rt.ref_type.clone(),
                &Variable::UnsafeRef(_) => rt.unsafe_ref_type.clone(),
                &Variable::RustObject(_) => rt.rust_object_type.clone(),
            };
            rt.stack.push(v);
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "debug" => {
            rt.push_fn(call.name.clone(), st, lc);
            println!("Stack {:#?}", rt.stack);
            println!("Locals {:#?}", rt.local_stack);
            rt.pop_fn(call.name.clone());
            Expect::Nothing
        }
        "backtrace" => {
            rt.push_fn(call.name.clone(), st, lc);
            println!("{:#?}", rt.call_stack);
            rt.pop_fn(call.name.clone());
            Expect::Nothing
        }
        "load" => {
            use load;

            rt.push_fn(call.name.clone(), st + 1, lc);
            let v = match rt.stack.pop() {
                Some(v) => v,
                None => panic!("There is no value on the stack")
            };
            let v = match rt.resolve(&v) {
                &Variable::Text(ref text) => {
                    let mut module = Module::new();
                    load(text, &mut module).unwrap_or_else(|err| {
                        panic!("{}", err);
                    });
                    Variable::RustObject(Arc::new(Mutex::new(
                        module)))
                }
                _ => panic!("Expected text argument")
            };
            rt.stack.push(v);
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "load_source_imports" => {
            use load;

            rt.push_fn(call.name.clone(), st + 1, lc);
            let modules = match rt.stack.pop() {
                Some(v) => v,
                None => panic!("There is no value on the stack")
            };
            let source = match rt.stack.pop() {
                Some(v) => v,
                None => panic!("There is no value on the stack")
            };
            let mut module = Module::new();
            match rt.resolve(&modules) {
                &Variable::Array(ref array) => {
                    for it in array {
                        match rt.resolve(it) {
                            &Variable::RustObject(ref obj) => {
                                match obj.lock().unwrap().downcast_ref::<Module>() {
                                    Some(m) => {
                                        for f in m.functions.values() {
                                            module.register(f.clone())
                                        }
                                    }
                                    None => panic!("Expected `Module`")
                                }
                            }
                            _ => panic!("Expected Rust object")
                        }
                    }
                }
                _ => panic!("Expected array argument")
            }
            let v = match rt.resolve(&source) {
                &Variable::Text(ref text) => {
                    load(text, &mut module).unwrap_or_else(|err| {
                        panic!("{}", err);
                    });
                    Variable::RustObject(
                        Arc::new(Mutex::new(module)))
                }
                _ => panic!("Expected text argument")
            };
            rt.stack.push(v);
            rt.pop_fn(call.name.clone());
            Expect::Something
        }
        "call" => {
            rt.push_fn(call.name.clone(), st, lc);
            let args = match rt.stack.pop() {
                Some(v) => v,
                None => panic!("There is no value on the stack")
            };
            let fn_name = match rt.stack.pop() {
                Some(v) => v,
                None => panic!("There is no value on the stack")
            };
            let module = match rt.stack.pop() {
                Some(v) => v,
                None => panic!("There is no value on the stack")
            };
            let fn_name = match rt.resolve(&fn_name) {
                &Variable::Text(ref text) => text.clone(),
                _ => panic!("Expected text argument")
            };
            let args = match rt.resolve(&args) {
                &Variable::Array(ref arr) => arr.clone(),
                _ => panic!("Expected array argument")
            };
            let obj = match rt.resolve(&module) {
                &Variable::RustObject(ref obj) => obj.clone(),
                _ => panic!("Expected rust object")
            };

            match obj.lock().unwrap()
                .downcast_ref::<Module>() {
                Some(m) => {
                    match m.functions.get(&fn_name) {
                        Some(ref f) => {
                            if f.args.len() != args.len() {
                                panic!("Expected `{}` arguments, found `{}`",
                                    f.args.len(), args.len())
                            }
                        }
                        None => panic!("Could not find function `{}`", fn_name)
                    }
                    let call = ast::Call {
                        name: fn_name.clone(),
                        args: args.into_iter().map(|arg|
                            ast::Expression::Variable(arg)).collect()
                    };
                    rt.call(&call, &m);
                }
                None => panic!("Expected `Vec<ast::Function>`")
            }

            rt.pop_fn(call.name.clone());
            Expect::Nothing
        }
        _ => panic!("Unknown function `{}`", call.name)
    };
    (expect, Flow::Continue)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ArgConstraint {
    Arg(usize),
    Return,
    Default,
}

#[derive(Debug, Copy, Clone)]
pub struct Intrinsic {
    pub arg_constraints: &'static [ArgConstraint],
    pub returns: bool,
}

static PRINTLN: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: false
};

static PRINT: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: false
};

static CLONE: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: false
};

static DEBUG: Intrinsic = Intrinsic {
    arg_constraints: &[],
    returns: false
};

static BACKTRACE: Intrinsic = Intrinsic {
    arg_constraints: &[],
    returns: false
};

static SLEEP: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: false
};

static ROUND: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static RANDOM: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static READ_NUMBER: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static READ_LINE: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static TRIM_RIGHT: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static LEN: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static PUSH: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default, ArgConstraint::Arg(0)],
    returns: false
};

static SQRT: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static ASIN: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static SIN: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static COS: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static ACOS: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static TAN: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static ATAN: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static EXP: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static LN: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static LOG2: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static LOG10: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static TO_STRING: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static TYPEOF: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static LOAD: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default],
    returns: true
};

static LOAD_SOURCE_IMPORTS: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default; 2],
    returns: true
};

static CALL: Intrinsic = Intrinsic {
    arg_constraints: &[ArgConstraint::Default; 3],
    returns: true
};
