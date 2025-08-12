use russcip::prelude::*;

// Problem: A factory produces two products (A and B)
// Product A: profit = $40, requires 2 hours labor, 1 unit material
// Product B: profit = $30, requires 1 hour labor, 2 units material
// Constraints: max 100 hours labor, max 80 units material
// Goal: maximize profit

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut model = Model::new()
        .hide_output()
        .include_default_plugins()
        .create_prob("production_planning")
        .set_obj_sense(ObjSense::Maximize);

    let var_a = model.add_var(0., f64::INFINITY, 40., "product_A", VarType::Integer);
    let var_b = model.add_var(0., f64::INFINITY, 30., "product_B", VarType::Integer);

    let _labor_constraint = model.add_cons(
        vec![&var_a, &var_b], 
        &[2.0, 1.0], 
        -f64::INFINITY, 
        100.0, 
        "labor_constraint"
    );

    let _material_constraint = model.add_cons(
        vec![&var_a, &var_b],
        &[1.0, 2.0],
        -f64::INFINITY,
        80.0,
        "material_constraint"
    );

    let solved_model = model.solve();

    match solved_model.status() {
        Status::Optimal => {
            let sol = solved_model.best_sol().unwrap();
            let a_value = sol.val(&var_a);
            let b_value = sol.val(&var_b);
            let objective_value = solved_model.obj_val();
            println!("Optimal solution found!");
            println!("Product A: {}", a_value);
            println!("Product B: {}", b_value);
            println!("Objective value: {}", objective_value);
        }
        _ => {
            return Err(format!("Solver finished with status: {:?}", solved_model.status()).into());
        }
    }

    Ok(())
}
