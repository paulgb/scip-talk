use anyhow::Result;
use russcip::{Model, ObjSense, ProblemOrSolving, Status, VarType, WithSolutions};
use image::{ImageBuffer, Rgb, RgbImage};

pub fn main() {
    // read a vector of integers from argv
    let args: Vec<String> = std::env::args().collect();
    let mut nums: Vec<i32> = parse_shorthand_args(&args[1..]);
    println!("Numbers: {:?}", nums);

    // sort nums in ascending order
    nums.sort();

    // generate pairings
    let pairings = generate_pairings(&nums).unwrap();

    let mut pairings_by_sender: Vec<Vec<usize>> = Vec::new();
    let mut pairings_by_receiver: Vec<Vec<usize>> = Vec::new();
    for _ in 0..nums.len() {
        pairings_by_sender.push(Vec::new());
        pairings_by_receiver.push(Vec::new());
    }

    for (i, j) in &pairings {
        pairings_by_sender[*i].push(*j);
        pairings_by_receiver[*j].push(*i);
    }

    for i in 0..nums.len() {
        println!("Participant {} requests {} cards (actual sent: {}, received: {})", i, nums[i], pairings_by_sender[i].len(), pairings_by_receiver[i].len());

        for j in &pairings_by_sender[i] {
            println!("send: {}", j);
        }

        for j in &pairings_by_receiver[i] {
            println!("receive: {}", j);
        }
    }

    println!("Total number of participants: {}", nums.len());
    println!("Total number of pairings: {}", pairings.len());

    let filename = "solution.png";
    
    // Create visualization
    if let Err(e) = visualize_solution_matrix(&pairings, &nums, &filename) {
        eprintln!("Failed to create visualization: {}", e);
    }
}

/// Parse command line arguments that support shorthand notation.
/// Examples:
/// - "3" -> [3]
/// - "3x4" -> [3, 3, 3, 3]
/// - "1x3 2x3 3x4" -> [1, 1, 1, 2, 2, 2, 3, 3, 3, 3]
fn parse_shorthand_args(args: &[String]) -> Vec<i32> {
    let mut result = Vec::new();
    
    for arg in args {
        if let Some(x_pos) = arg.find('x') {
            // Parse "NxM" format
            let (num_str, count_str) = arg.split_at(x_pos);
            let count_str = &count_str[1..]; // Skip the 'x'
            
            let num: i32 = num_str.parse().expect(&format!("Invalid number: {}", num_str));
            let count: usize = count_str.parse().expect(&format!("Invalid count: {}", count_str));
            
            for _ in 0..count {
                result.push(num);
            }
        } else {
            // Parse single number
            let num: i32 = arg.parse().expect(&format!("Invalid number: {}", arg));
            result.push(num);
        }
    }
    
    result
}

pub fn generate_pairings(cards_for_participant: &Vec<i32>) -> Result<Vec<(usize, usize)>> {
    let n = cards_for_participant.len();

    let mut model = Model::new()
        .hide_output()
        .include_default_plugins()
        .create_prob("pairings")
        .set_obj_sense(ObjSense::Maximize);

    // x[i][j] is 1 if person i sends a card to person j
    let mut x = Vec::new();
    for _ in 0..n {
        let mut row = Vec::new();
        for _ in 0..n {
            row.push(model.add_var(0., 1., 1., "adjacency", VarType::Binary));
        }
        x.push(row);
    }

    // Nobody sends a card to themself.
    for i in 0..n {
        model.add_cons(vec![&x[i][i]], &[1.], 0., 0., "no_self_exchange");
    }

    // Nobody sends a card to someone who sent a card to them.
    for i in 0..n {
        for j in (i + 1)..n {
            model.add_cons(
                vec![&x[i][j], &x[j][i]],
                &[1., 1.],
                0.,
                1.,
                "no_mutual_exchange",
            );
        }
    }

    // Nobody sends more cards than they signed up for.
    for i in 0..n {
        let num_cards = cards_for_participant[i];
        model.add_cons(
            x[i].iter().collect(),
            &vec![1.0; n],
            0.,
            num_cards as f64,
            "num_cards",
        );
    }

    // Everyone receives a card for every card they send.
    for i in 0..n {
        // Collect variables representing cards that i sends, and give them a coefficient of +1.
        let mut vars: Vec<_> = x[i].iter().cloned().collect();
        let mut coefs = vec![1.0; n];
        // Collect variables representing cards that i receives, and give them a coefficient of -1.
        vars.extend(x.iter().map(|row| row[i].clone()));
        coefs.extend_from_slice(vec![-1.0; n].as_ref());
        model.add_cons(vars.iter().collect(), &coefs, 0., 0., "card_balance");
    }

    println!("Attempting to solve...");
    let solved_model = model.solve();
    if solved_model.status() != Status::Optimal {
        anyhow::bail!("Optimal solution not found");
    }

    let obj_val = solved_model.obj_val();
    println!("Solved. Objective value: {}", obj_val);

    let sol = solved_model.best_sol().unwrap();

    let mut result: Vec<(usize, usize)> = Vec::new();
    for i in 0..n {
        for j in 0..n {
            if sol.val(&x[i][j]) >= 0.9 {
                result.push((i, j));
            }
        }
    }

    Ok(result)
}

/// Visualize the solution matrix as a PNG image where each cell is a 9x9 square with 1px white borders
pub fn visualize_solution_matrix(
    pairings: &Vec<(usize, usize)>,
    nums: &Vec<i32>,
    filename: &str,
) -> Result<()> {
    let n = nums.len();
    let cell_size = 9;
    let border_size = 1;
    // Image size: n cells of size cell_size + (n+1) borders of size border_size
    let image_size = n * cell_size + (n + 1) * border_size;
    
    // Create a new RGB image with white background (for borders)
    let mut img: RgbImage = ImageBuffer::new(image_size as u32, image_size as u32);
    let white = Rgb([255, 255, 255]);
    
    // Fill the entire image with white (this creates the border effect)
    for y in 0..image_size {
        for x in 0..image_size {
            img.put_pixel(x as u32, y as u32, white);
        }
    }
    
    // Count how many pairings each row (sender) and column (receiver) has
    let mut row_counts = vec![0; n];
    let mut col_counts = vec![0; n];
    for (i, j) in pairings {
        row_counts[*i] += 1;
        col_counts[*j] += 1;
    }
    
    // Find the maximum counts for normalization
    let max_row_count = *row_counts.iter().max().unwrap_or(&1);
    let max_col_count = *col_counts.iter().max().unwrap_or(&1);
    
    // Define colors
    let blue = Rgb([0, 100, 200]);   // Has pairing
    
    // Fill each cell
    for row in 0..n {
        for col in 0..n {
            // Calculate cell position accounting for borders
            let start_x = border_size + col * (cell_size + border_size);
            let start_y = border_size + row * (cell_size + border_size);
            
            // Check if this cell has a pairing
            let has_pairing = pairings.iter().any(|(i, j)| *i == row && *j == col);
            
            if has_pairing {
                // Fill entire cell with blue
                for dy in 0..cell_size {
                    for dx in 0..cell_size {
                        let x = start_x + dx;
                        let y = start_y + dy;
                        if x < image_size && y < image_size {
                            img.put_pixel(x as u32, y as u32, blue);
                        }
                    }
                }
            } else {
                let color_scale = 128.0;
                // Split cell diagonally into two triangles
                // Calculate grey tones for row (bottom-left triangle) and column (top-right triangle)
                let row_activity_ratio = row_counts[row] as f32 / max_row_count as f32;
                let row_grey_value = (256.0 - (row_activity_ratio * color_scale)) as u8; // 240 down to 200
                let row_color = Rgb([row_grey_value, row_grey_value, row_grey_value]);
                
                let col_activity_ratio = col_counts[col] as f32 / max_col_count as f32;
                let col_grey_value = (256.0 - (col_activity_ratio * color_scale)) as u8; // 240 down to 200
                let col_color = Rgb([col_grey_value, col_grey_value, col_grey_value]);
                
                // Fill the 9x9 square with diagonal split
                for dy in 0..cell_size {
                    for dx in 0..cell_size {
                        let x = start_x + dx;
                        let y = start_y + dy;
                        if x < image_size && y < image_size {
                            // Determine which triangle this pixel is in
                            // Top-left to bottom-right diagonal: if dx >= dy, it's top-right triangle (column-based)
                            // if dx < dy, it's bottom-left triangle (row-based)
                            let color = if dx >= dy {
                                col_color  // Top-right triangle: column activity
                            } else {
                                row_color  // Bottom-left triangle: row activity
                            };
                            img.put_pixel(x as u32, y as u32, color);
                        }
                    }
                }
            }
        }
    }
    
    // Save the image
    img.save(filename)?;
    println!("Matrix visualization saved as: {}", filename);
    
    Ok(())
}
