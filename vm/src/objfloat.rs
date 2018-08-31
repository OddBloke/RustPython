use super::objtype;
use super::pyobject::{
    AttributeProtocol, FromPyObject, PyContext, PyFuncArgs, PyObjectRef, PyResult, TypeProtocol,
};
use super::vm::VirtualMachine;

fn str(vm: &mut VirtualMachine, args: PyFuncArgs) -> Result<PyObjectRef, PyObjectRef> {
    arg_check!(vm, args, required = [(float, Some(vm.ctx.float_type()))]);
    let v = f64::from_pyobject(float);
    Ok(vm.new_str(v.to_string()))
}

fn float_add(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(
        vm,
        args,
        required = [(i, Some(vm.ctx.float_type())), (i2, None)]
    );

    let v1 = f64::from_pyobject(i);
    if objtype::isinstance(i2.clone(), vm.ctx.float_type()) {
        Ok(vm.ctx.new_float(v1 + f64::from_pyobject(i2)))
    } else if objtype::isinstance(i2.clone(), vm.ctx.int_type()) {
        Ok(vm.ctx.new_float(v1 + f64::from_pyobject(i2)))
    } else {
        Err(vm.new_type_error(format!("Cannot add {:?} and {:?}", i, i2)))
    }
}

fn float_sub(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(
        vm,
        args,
        required = [(i, Some(vm.ctx.float_type())), (i2, None)]
    );

    let v1 = f64::from_pyobject(i);
    if objtype::isinstance(i2.clone(), vm.ctx.float_type()) {
        Ok(vm.ctx.new_float(v1 - f64::from_pyobject(i2)))
    } else if objtype::isinstance(i2.clone(), vm.ctx.int_type()) {
        Ok(vm.ctx.new_float(v1 - f64::from_pyobject(i2)))
    } else {
        Err(vm.new_type_error(format!("Cannot add {:?} and {:?}", i, i2)))
    }
}

fn float_pow(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(
        vm,
        args,
        required = [(i, Some(vm.ctx.float_type())), (i2, None)]
    );

    let v1 = f64::from_pyobject(i);
    if objtype::isinstance(i2.clone(), vm.ctx.float_type()) {
        let result = v1.powf(f64::from_pyobject(i2));
        Ok(vm.ctx.new_float(result))
    } else if objtype::isinstance(i2.clone(), vm.ctx.int_type()) {
        let result = v1.powf(f64::from_pyobject(i2));
        Ok(vm.ctx.new_float(result))
    } else {
        Err(vm.new_type_error(format!("Cannot add {:?} and {:?}", i, i2)))
    }
}

pub fn init(context: &PyContext) {
    let ref float_type = context.float_type;
    float_type.set_attr("__add__", context.new_rustfunc(float_add));
    float_type.set_attr("__pow__", context.new_rustfunc(float_pow));
    float_type.set_attr("__str__", context.new_rustfunc(str));
    float_type.set_attr("__sub__", context.new_rustfunc(float_sub));
    float_type.set_attr("__repr__", context.new_rustfunc(str));
}
