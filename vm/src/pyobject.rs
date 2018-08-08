use super::bytecode;
use super::objint;
use super::objtype;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::ops::{Add, Div, Mul, Sub, Rem};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use super::vm::VirtualMachine;

/* Python objects and references.

Okay, so each python object itself is an class itself (PyObject). Each
python object can have several references to it (PyObjectRef). These
references are Rc (reference counting) rust smart pointers. So when
all references are destroyed, the object itself also can be cleaned up.
Basically reference counting, but then done by rust.

*/

/*
 * Good reference: https://github.com/ProgVal/pythonvm-rust/blob/master/src/objects/mod.rs
 */

/*
The PyRef type implements
https://doc.rust-lang.org/std/cell/index.html#introducing-mutability-inside-of-something-immutable
*/
pub type PyRef<T> = Rc<RefCell<T>>;
pub type PyObjectRef = PyRef<PyObject>;
pub type PyResult = Result<PyObjectRef, PyObjectRef>; // A valid value, or an exception

/*
impl fmt::Display for PyObjectRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Obj {:?}", self)
    }
}*/

#[derive(Debug)]
pub struct PyContext {
    pub type_type: PyObjectRef,
    pub int_type: PyObjectRef,
    pub list_type: PyObjectRef,
    pub tuple_type: PyObjectRef,
    pub dict_type: PyObjectRef,
}

/*
 * So a scope is a linked list of scopes.
 * When a name is looked up, it is check in its scope.
 */
#[derive(Debug)]
pub struct Scope {
    pub locals: PyObjectRef, // Variables
    pub parent: Option<PyObjectRef>,  // Parent scope
}

// Basic objects:
impl PyContext {
    pub fn new() -> PyContext {
        let type_type = objtype::create_type();
        let int_type = objint::create_type(type_type.clone());
        // TODO: How to represent builtin types?
        let list_type = type_type.clone();
        let tuple_type = type_type.clone();
        let dict_type = type_type.clone();
        // let str_type = objstr::make_type();
        PyContext {
            type_type: type_type,
            int_type: int_type,
            list_type: list_type,
            tuple_type: tuple_type,
            dict_type: dict_type,
        }
    }

    pub fn new_int(&self, i: i32) -> PyObjectRef {
        PyObject::new(PyObjectKind::Integer { value: i }, self.type_type.clone())
    }

    pub fn new_str(&self, s: String) -> PyObjectRef {
        PyObject::new(PyObjectKind::String { value: s }, self.type_type.clone())
    }

    pub fn new_bool(&self, b: bool) -> PyObjectRef {
        PyObject::new(PyObjectKind::Boolean { value: b }, self.type_type.clone())
    }

    pub fn new_tuple(&self) -> PyObjectRef {
        PyObject::new(
            PyObjectKind::Tuple {
                elements: Vec::new(),
            },
            self.type_type.clone(),
        )
    }

    pub fn new_list(&self, elements: Option<Vec<PyObjectRef>>) -> PyObjectRef {
        PyObject::new(
            PyObjectKind::List {
                elements: elements.unwrap_or(Vec::new()),
            },
            self.type_type.clone(),
        )
    }

    pub fn new_dict(&self) -> PyObjectRef {
        PyObject::new(
            PyObjectKind::Dict {
                elements: HashMap::new(),
            },
            self.type_type.clone(),
        )
    }

    pub fn new_scope(&self, parent: Option<PyObjectRef>) -> PyObjectRef {
        let locals = self.new_dict();
        let scope = Scope {
            locals: locals,
            parent: parent,
        };
        PyObject { kind: PyObjectKind::Scope { scope: scope }, typ: None }.into_ref()
    }

    pub fn new_module(&self, name: &String, scope: PyObjectRef) -> PyObjectRef {
        PyObject::new(
            PyObjectKind::Module {
                name: name.clone(),
                dict: scope.clone(),
            },
            self.type_type.clone(),
        )
    }

    pub fn new_rustfunc(&self, function: RustPyFunc) -> PyObjectRef {
        PyObject::new(
            PyObjectKind::RustFunction { function: function },
            self.type_type.clone(),
        )
    }

    pub fn new_class(&self, name: String) -> PyObjectRef {
        PyObject::new(PyObjectKind::Class { name: name }, self.type_type.clone())
    }

    /* TODO: something like this?
    pub fn new_instance(&self, name: String) -> PyObjectRef {
        PyObject::new(PyObjectKind::Class { name: name }, self.type_type.clone())
    }
    */
}

pub struct PyObject {
    pub kind: PyObjectKind,
    pub typ: Option<PyObjectRef>,
    // pub dict: HashMap<String, PyObjectRef>, // __dict__ member
}

impl Default for PyObject {
    fn default() -> PyObject {
        PyObject {
            kind: PyObjectKind::None,
            typ: None,
            // dict: HashMap::new(),
        }
    }
}

impl Hash for PyObject {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
    }
}

pub trait IdProtocol {
    fn get_id(&self) -> usize;
}

impl IdProtocol for PyObjectRef {
    fn get_id(&self) -> usize {
        self.as_ptr() as usize
    }
}

pub trait ParentProtocol {
    fn has_parent(&self) -> bool;
    fn get_parent(&self) -> PyObjectRef;
}

impl ParentProtocol for PyObjectRef {
    fn has_parent(&self) -> bool {
        match self.borrow().kind {
            PyObjectKind::Scope { ref scope } => match scope.parent {
                Some(_) => true,
                None => false,
            },
            _ => panic!("Only scopes have parent (not {:?}", self),
        }
    }

    fn get_parent(&self) -> PyObjectRef {
        match self.borrow().kind {
            PyObjectKind::Scope { ref scope } => match scope.parent {
                Some(ref value) => value.clone(),
                None => panic!("OMG"),
            },
            _ => panic!("TODO"),
        }
    }
}

pub trait DictProtocol {
    fn contains_key(&self, k: &String) -> bool;
    fn get_item(&self, k: &String) -> PyObjectRef;
    fn set_item(&mut self, k: &String, v: PyObjectRef);
}

impl DictProtocol for PyObjectRef {
    fn contains_key(&self, k: &String) -> bool {
        match self.borrow().kind {
            PyObjectKind::Dict { ref elements } => elements.contains_key(k),
            PyObjectKind::Module { name: _, ref dict } => dict.contains_key(k),
            PyObjectKind::Scope { ref scope } => scope.locals.contains_key(k),
            _ => panic!("TODO"),
        }
    }

    fn get_item(&self, k: &String) -> PyObjectRef {
        match self.borrow().kind {
            PyObjectKind::Dict { ref elements } => elements[k].clone(),
            PyObjectKind::Module { name: _, ref dict } => dict.get_item(k),
            PyObjectKind::Scope { ref scope } => scope.locals.get_item(k),
            _ => panic!("TODO"),
        }
    }

    fn set_item(&mut self, k: &String, v: PyObjectRef) {
        match self.borrow_mut().kind {
            PyObjectKind::Dict {
                elements: ref mut el,
            } => {
                el.insert(k.to_string(), v);
            },
            PyObjectKind::Module { name: _, ref mut dict } => dict.set_item(k, v),
            PyObjectKind::Scope { ref mut scope } => {
                scope.locals.set_item(k, v);
            },
            _ => panic!("TODO"),
        };
    }
}

impl fmt::Debug for PyObject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[PyObj {:?}]", self.kind)
    }
}

#[derive(Debug)]
pub struct PyFuncArgs {
    pub args: Vec<PyObjectRef>,
    // TODO: add kwargs here
}

type RustPyFunc = fn(vm: &mut VirtualMachine, PyFuncArgs) -> PyResult;

pub enum PyObjectKind {
    String {
        value: String,
    },
    Integer {
        value: i32,
    },
    Float {
        value: f64,
    },
    Boolean {
        value: bool,
    },
    List {
        elements: Vec<PyObjectRef>,
    },
    Tuple {
        elements: Vec<PyObjectRef>,
    },
    Dict {
        elements: HashMap<String, PyObjectRef>,
    },
    Iterator {
        position: usize,
        iterated_obj: PyObjectRef,
    },
    Slice {
        start: Option<i32>,
        stop: Option<i32>,
        step: Option<i32>,
    },
    NameError {
        // TODO: improve python object and type system
        name: String,
    },
    Code {
        code: bytecode::CodeObject,
    },
    Function {
        code: PyObjectRef,
        scope: PyObjectRef,
    },
    Scope {
        scope: Scope,
    },
    Module {
        name: String,
        dict: PyObjectRef,
    },
    None,
    Class {
        name: String,
        // dict: PyObjectRef,
    },
    RustFunction {
        function: RustPyFunc,
    },
}

impl fmt::Debug for PyObjectKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &PyObjectKind::String { ref value } => write!(f, "str \"{}\"", value),
            &PyObjectKind::Integer { ref value } => write!(f, "int {}", value),
            &PyObjectKind::Float { ref value } => write!(f, "float {}", value),
            &PyObjectKind::Boolean { ref value } => write!(f, "boolean {}", value),
            &PyObjectKind::List { elements: _ } => write!(f, "list"),
            &PyObjectKind::Tuple { elements: _ } => write!(f, "tuple"),
            &PyObjectKind::Dict { elements: _ } => write!(f, "dict"),
            &PyObjectKind::Iterator { position: _, iterated_obj: _ } => write!(f, "iterator"),
            &PyObjectKind::Slice { start: _, stop: _, step: _ } => write!(f, "slice"),
            &PyObjectKind::NameError { name: _ } => write!(f, "NameError"),
            &PyObjectKind::Code { ref code } => write!(f, "code: {:?}", code),
            &PyObjectKind::Function { code: _, scope: _ } => write!(f, "function"),
            &PyObjectKind::Module { name: _, dict: _ } => write!(f, "module"),
            &PyObjectKind::Scope { scope: _ } => write!(f, "scope"),
            &PyObjectKind::None => write!(f, "None"),
            &PyObjectKind::Class { name: _ } => write!(f, "class"),
            &PyObjectKind::RustFunction { function: _ } => write!(f, "rust function"),
        }
    }
}

impl Hash for PyObjectKind {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            &PyObjectKind::String { ref value } => value.hash(state),
            &PyObjectKind::Integer { ref value } => value.hash(state),
            &PyObjectKind::Boolean { ref value } => value.hash(state),
            &PyObjectKind::List { ref elements } | &PyObjectKind::Tuple { ref elements } => {
                for element in elements {
                    element.borrow().hash(state)
                }
            }
            &PyObjectKind::Iterator {
                ref position,
                ref iterated_obj,
            } => {
                position.hash(state);
                iterated_obj.borrow().hash(state)
            }
            &PyObjectKind::Slice {
                ref start,
                ref stop,
                ref step,
            } => {
                start.hash(state);
                stop.hash(state);
                step.hash(state)
            }
            &PyObjectKind::NameError { ref name } => name.hash(state),
            &PyObjectKind::Module { ref name, ref dict } => name.hash(state),
            &PyObjectKind::None => "None".hash(state),
            &PyObjectKind::Class { ref name } => name.hash(state),
            ref kind => panic!("hashing not yet supported for: {:?}", kind),
        }
    }
}

impl PyObject {
    pub fn new(kind: PyObjectKind, /* dict: PyObjectRef,*/ typ: PyObjectRef) -> PyObjectRef {
        PyObject {
            kind: kind,
            typ: Some(typ),
            // dict: HashMap::new(),  // dict,
        }.into_ref()
    }

    pub fn call(&self, vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
        match self.kind {
            PyObjectKind::RustFunction { ref function } => function(vm, args),
            _ => {
                println!("Not impl {:?}", self);
                panic!("Not impl");
            }
        }
    }

    pub fn str(&self) -> String {
        match self.kind {
            PyObjectKind::String { ref value } => value.clone(),
            PyObjectKind::Integer { ref value } => format!("{:?}", value),
            PyObjectKind::Boolean { ref value } => format!("{:?}", value),
            PyObjectKind::List { ref elements } => format!(
                "[{}]",
                elements
                    .iter()
                    .map(|elem| elem.borrow().str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            PyObjectKind::Tuple { ref elements } => if elements.len() == 1 {
                format!("({},)", elements[0].borrow().str())
            } else {
                format!(
                    "({})",
                    elements
                        .iter()
                        .map(|elem| elem.borrow().str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            },
            PyObjectKind::Dict { ref elements } => format!(
                "{{ {} }}",
                elements
                    .iter()
                    .map(|elem| format!("{}: {}", elem.0, elem.1.borrow().str()))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            PyObjectKind::None => String::from("None"),
            PyObjectKind::Class { ref name } => format!("<class '{}'>", name),
            PyObjectKind::Code { code: _ } => format!("<code>"),
            PyObjectKind::Function { code: _, scope: _ } => format!("<func>"),
            PyObjectKind::RustFunction { function: _ } => format!("<rustfunc>"),
            PyObjectKind::Module { ref name, ref dict } => format!("<module '{}'>", name),
            PyObjectKind::Scope { ref scope } => format!("<scope '{:?}'>", scope),
            PyObjectKind::Slice {
                ref start,
                ref stop,
                ref step,
            } => format!("<slice '{:?}:{:?}:{:?}'>", start, stop, step),
            PyObjectKind::Iterator {
                ref position,
                ref iterated_obj,
            } => format!(
                "<iter pos {} in {}>",
                position,
                iterated_obj.borrow_mut().str()
            ),
            _ => {
                println!("Not impl {:?}", self);
                panic!("Not impl");
            }
        }
    }

    // Implement iterator protocol:
    pub fn nxt(&mut self) -> Option<PyObjectRef> {
        match self.kind {
            PyObjectKind::Iterator {
                ref mut position,
                iterated_obj: ref iterated_obj_ref,
            } => {
                let iterated_obj = &*iterated_obj_ref.borrow_mut();
                match iterated_obj.kind {
                    PyObjectKind::List { ref elements } => {
                        if *position < elements.len() {
                            let obj_ref = elements[*position].clone();
                            *position += 1;
                            Some(obj_ref)
                        } else {
                            None
                        }
                    }
                    _ => {
                        panic!("NOT IMPL");
                    }
                }
            }
            _ => {
                panic!("NOT IMPL");
            }
        }
    }

    // Move this object into a reference object, transferring ownership.
    pub fn into_ref(self) -> PyObjectRef {
        Rc::new(RefCell::new(self))
    }
}

impl<'a> Add<&'a PyObject> for &'a PyObject {
    type Output = PyObjectKind;

    fn add(self, rhs: &'a PyObject) -> Self::Output {
        match self.kind {
            PyObjectKind::Integer { value: ref value1 } => match &rhs.kind {
                PyObjectKind::Integer { value: ref value2 } => PyObjectKind::Integer {
                    value: value1 + value2,
                },
                _ => {
                    panic!("NOT IMPL");
                }
            },
            PyObjectKind::String { value: ref value1 } => match rhs.kind {
                PyObjectKind::String { value: ref value2 } => PyObjectKind::String {
                    value: format!("{}{}", value1, value2),
                },
                _ => {
                    panic!("NOT IMPL");
                }
            },
            PyObjectKind::List { elements: ref e1 } => match rhs.kind {
                PyObjectKind::List { elements: ref e2 } => PyObjectKind::List {
                    elements: e1.iter().chain(e2.iter()).map(|e| e.clone()).collect(),
                },
                _ => {
                    panic!("NOT IMPL");
                }
            },
            _ => {
                // TODO: Lookup __add__ method in dictionary?
                panic!("NOT IMPL");
            }
        }
    }
}

impl<'a> Sub<&'a PyObject> for &'a PyObject {
    type Output = PyObjectKind;

    fn sub(self, rhs: &'a PyObject) -> Self::Output {
        match self.kind {
            PyObjectKind::Integer { value: value1 } => match rhs.kind {
                PyObjectKind::Integer { value: value2 } => PyObjectKind::Integer {
                    value: value1 - value2,
                },
                _ => {
                    panic!("NOT IMPL");
                }
            },
            _ => {
                panic!("NOT IMPL");
            }
        }
    }
}

impl<'a> Mul<&'a PyObject> for &'a PyObject {
    type Output = PyObjectKind;

    fn mul(self, rhs: &'a PyObject) -> Self::Output {
        match self.kind {
            PyObjectKind::Integer { value: value1 } => match rhs.kind {
                PyObjectKind::Integer { value: value2 } => PyObjectKind::Integer {
                    value: value1 * value2,
                },
                _ => {
                    panic!("NOT IMPL");
                }
            },
            PyObjectKind::String { value: ref value1 } => match rhs.kind {
                PyObjectKind::Integer { value: value2 } => {
                    let mut result = String::new();
                    for _x in 0..value2 {
                        result.push_str(value1.as_str());
                    }
                    PyObjectKind::String { value: result }
                }
                _ => {
                    panic!("NOT IMPL");
                }
            },
            _ => {
                panic!("NOT IMPL");
            }
        }
    }
}

impl<'a> Div<&'a PyObject> for &'a PyObject {
    type Output = PyObjectKind;

    fn div(self, rhs: &'a PyObject) -> Self::Output {
        match (&self.kind, &rhs.kind) {
            (PyObjectKind::Integer { value: value1 }, PyObjectKind::Integer { value: value2 }) => {
                PyObjectKind::Integer {
                    value: value1 / value2,
                }
            }
            _ => {
                panic!("NOT IMPL");
            }
        }
    }
}

impl<'a> Rem<&'a PyObject> for &'a PyObject {
    type Output = PyObjectKind;

    fn rem(self, rhs: &'a PyObject) -> Self::Output {
        match (&self.kind, &rhs.kind) {
            (PyObjectKind::Integer { value: value1 }, PyObjectKind::Integer { value: value2 }) => {
                PyObjectKind::Integer {
                    value: value1 % value2,
                }
            }
            (kind1, kind2) => {
                unimplemented!("% not implemented for kinds: {:?} {:?}", kind1, kind2);
            }
        }
    }
}

// impl<'a> PartialEq<&'a PyObject> for &'a PyObject {
impl PartialEq for PyObject {
    fn eq(&self, other: &PyObject) -> bool {
        match (&self.kind, &other.kind) {
            (
                PyObjectKind::Integer { value: ref v1i },
                PyObjectKind::Integer { value: ref v2i },
            ) => v2i == v1i,
            (PyObjectKind::String { value: ref v1i }, PyObjectKind::String { value: ref v2i }) => {
                *v2i == *v1i
            }
            /*
            (&NativeType::Float(ref v1f), &NativeType::Float(ref v2f)) => {
                curr_frame.stack.push(Rc::new(NativeType::Boolean(v2f == v1f)));
            },
            */
            (PyObjectKind::String { value: ref v1s }, &PyObjectKind::String { value: ref v2s }) => {
                v2s == v1s
            }
            (PyObjectKind::List { elements: ref l1 }, PyObjectKind::List { elements: ref l2 })
            | (
                PyObjectKind::Tuple { elements: ref l1 },
                PyObjectKind::Tuple { elements: ref l2 },
            ) => {
                if l1.len() == l2.len() {
                    Iterator::zip(l1.iter(), l2.iter()).all(|elem| elem.0 == elem.1)
                } else {
                    false
                }
            }
            _ => panic!(
                "TypeError in COMPARE_OP: can't compare {:?} with {:?}",
                self, other
            ),
        }
    }
}

impl Eq for PyObject {}

impl PartialOrd for PyObject {
    fn partial_cmp(&self, other: &PyObject) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PyObject {
    fn cmp(&self, other: &PyObject) -> Ordering {
        match (&self.kind, &other.kind) {
            (PyObjectKind::Integer { value: v1 }, PyObjectKind::Integer { value: ref v2 }) => {
                v1.cmp(v2)
            }
            _ => panic!("Not impl"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PyContext, PyObjectKind};

    #[test]
    fn test_add_py_integers() {
        let ctx = PyContext::new();
        let a = ctx.new_int(33);
        let b = ctx.new_int(12);
        let c = &*a.borrow() + &*b.borrow();
        match c {
            PyObjectKind::Integer { value } => assert_eq!(value, 45),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_multiply_str() {
        let ctx = PyContext::new();
        let a = ctx.new_str(String::from("Hello "));
        let b = ctx.new_int(4);
        let c = &*a.borrow() * &*b.borrow();
        match c {
            PyObjectKind::String { value } => {
                assert_eq!(value, String::from("Hello Hello Hello Hello "))
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_type_type() {
        let ctx = PyContext::new();
    }
}
