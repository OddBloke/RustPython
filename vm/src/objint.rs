use super::objtype;
use super::pyobject::{
    AttributeProtocol, FromPyObject, PyContext, PyFuncArgs, PyObjectRef, PyResult, TypeProtocol,
};
use super::vm::VirtualMachine;

fn str(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(vm, args, required = [(int, Some(vm.ctx.int_type()))]);
    let v = i32::from_pyobject(int);
    Ok(vm.new_str(v.to_string()))
}

fn int_add(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(
        vm,
        args,
        required = [(i, Some(vm.ctx.int_type())), (i2, None)]
    );
    if objtype::isinstance(i2.clone(), vm.ctx.int_type()) {
        Ok(vm
            .ctx
            .new_int(i32::from_pyobject(i) + i32::from_pyobject(i2)))
    } else if objtype::isinstance(i2.clone(), vm.ctx.float_type()) {
        Ok(vm
            .ctx
            .new_float(f64::from_pyobject(i) + f64::from_pyobject(i2)))
    } else {
        Err(vm.new_type_error(format!("Cannot add {:?} and {:?}", i, i2)))
    }
}

fn int_sub(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(
        vm,
        args,
        required = [(i, Some(vm.ctx.int_type())), (i2, None)]
    );
    if objtype::isinstance(i2.clone(), vm.ctx.int_type()) {
        Ok(vm
            .ctx
            .new_int(i32::from_pyobject(i) - i32::from_pyobject(i2)))
    } else {
        Err(vm.new_type_error(format!("Cannot substract {:?} and {:?}", i, i2)))
    }
}

fn int_mul(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(
        vm,
        args,
        required = [(i, Some(vm.ctx.int_type())), (i2, None)]
    );
    if objtype::isinstance(i2.clone(), vm.ctx.int_type()) {
        Ok(vm
            .ctx
            .new_int(i32::from_pyobject(i) * i32::from_pyobject(i2)))
    } else {
        Err(vm.new_type_error(format!("Cannot multiply {:?} and {:?}", i, i2)))
    }
}

fn int_truediv(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(
        vm,
        args,
        required = [(i, Some(vm.ctx.int_type())), (i2, None)]
    );
    if objtype::isinstance(i2.clone(), vm.ctx.int_type()) {
        Ok(vm
            .ctx
            .new_float(f64::from_pyobject(i) / f64::from_pyobject(i2)))
    } else if objtype::isinstance(i2.clone(), vm.ctx.float_type()) {
        Ok(vm
            .ctx
            .new_float(f64::from_pyobject(i) / f64::from_pyobject(i2)))
    } else {
        Err(vm.new_type_error(format!("Cannot multiply {:?} and {:?}", i, i2)))
    }
}

fn int_mod(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(
        vm,
        args,
        required = [(i, Some(vm.ctx.int_type())), (i2, None)]
    );
    if objtype::isinstance(i2.clone(), vm.ctx.int_type()) {
        Ok(vm
            .ctx
            .new_int(i32::from_pyobject(i) % i32::from_pyobject(i2)))
    } else {
        Err(vm.new_type_error(format!("Cannot modulo {:?} and {:?}", i, i2)))
    }
}

fn int_pow(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(
        vm,
        args,
        required = [(i, Some(vm.ctx.int_type())), (i2, None)]
    );
    let v1 = i32::from_pyobject(i);
    if objtype::isinstance(i2.clone(), vm.ctx.int_type()) {
        let v2 = i32::from_pyobject(i2);
        Ok(vm.ctx.new_int(v1.pow(v2 as u32)))
    } else if objtype::isinstance(i2.clone(), vm.ctx.float_type()) {
        let v2 = f64::from_pyobject(i2);
        Ok(vm.ctx.new_float((v1 as f64).powf(v2)))
    } else {
        Err(vm.new_type_error(format!("Cannot modulo {:?} and {:?}", i, i2)))
    }
}

pub fn init(context: &PyContext) {
    let ref int_type = context.int_type;
    int_type.set_attr("__add__", context.new_rustfunc(int_add));
    int_type.set_attr("__mod__", context.new_rustfunc(int_mod));
    int_type.set_attr("__mul__", context.new_rustfunc(int_mul));
    int_type.set_attr("__pow__", context.new_rustfunc(int_pow));
    int_type.set_attr("__repr__", context.new_rustfunc(str));
    int_type.set_attr("__str__", context.new_rustfunc(str));
    int_type.set_attr("__sub__", context.new_rustfunc(int_sub));
    int_type.set_attr("__truediv__", context.new_rustfunc(int_truediv));
}
