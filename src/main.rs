use std::f32::consts::PI;
use minifb::{Window, WindowOptions};

const THETA_ITER: f32 = 0.01;

const WIDTH: usize = 600;
const HEIGHT: usize = 600;

fn dot_product(m: &Vec<Vec<f32>>, n: &Vec<Vec<f32>>) -> Vec<Vec<f32>> {
    let mut res = vec![vec![0.0]; 4];

    for i in 0..m.len() {
        for j in 0..n[0].len() {
            for k in 0..m[0].len() {
                res[i][j] += m[i][k] * n[k][j];
            }
        }
    }

    res
}

fn draw_line(buffer: &mut Vec<u32>, p0: (i32, i32), p1: (i32, i32)) {
    let (mut x0, mut y0) = (p0.0 as i64, p0.1 as i64);
    let (x1, y1) = (p1.0 as i64, p1.1 as i64);

    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();

    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };

    let mut err = dx - dy;

    loop {
        // Set pixel at (x0, y0)
        if x0 >= 0 && y0 >= 0 && (x0 as usize) < WIDTH && (y0 as usize) < buffer.len() / WIDTH {
            let idx = (y0 as usize) * WIDTH + (x0 as usize);
            buffer[idx] = ((255 as u32) << 16) | ((255 as u32) << 8) | (255 as u32);
        }

        if x0 == x1 && y0 == y1 {
            break;
        }

        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x0 += sx;
        }
        if e2 < dx {
            err += dx;
            y0 += sy;
        }
    }
}

fn draw(buffer: &mut Vec<u32>, vertex: &Vec<(i32, i32)>, faces: &Vec<Vec<i32>>) {
    // Set canvas full black
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let idx = y * WIDTH + x;
            buffer[idx] = ((0 as u32) << 16) | ((0 as u32) << 8) | (0 as u32);
        }
    }
    
    for i in 0..faces.len() {
        for j in 0..faces[i].len() {
            let start_idx = faces[i][j];
            let end_idx = faces[i][(j + 1) % faces[i].len()];

            draw_line(buffer, vertex[start_idx as usize], vertex[end_idx as usize]);
        }
    }
}

fn main() {
    // Angle of view for the prism
    let mut theta = 0.0;

    // Initial prism points
    let mut prism = vec![
        vec![
            vec![-1.0],
            vec![-1.0],
            vec![-1.0],
            vec![1.0],
        ],
        vec![
            vec![1.0],
            vec![-1.0],
            vec![-1.0],
            vec![1.0],
        ],
        vec![
            vec![1.0],
            vec![-1.0],
            vec![1.0],
            vec![1.0],
        ],
        vec![
            vec![-1.0],
            vec![-1.0],
            vec![1.0],
            vec![1.0],
        ],
        vec![
            vec![0.0],
            vec![2.0],
            vec![0.0],
            vec![1.0],
        ]
    ];

    // Storing initial position of the prism
    let prism_original = prism.clone();

    // Faces of the prism, stored as collection of point's index in the prism
    let faces = vec![
        vec![0, 1, 2, 3],   // bottom   face
        vec![0, 1, 4],      // back     face
        vec![1, 2, 4],      // right    face
        vec![2, 3, 4],      // front    face
        vec![3, 0, 4]       // left     face
    ];

    // Setting up the values for the perspective matrix
    let fov:    f32 = PI / 2.0;
    let aspect: f32 = (WIDTH as f32) / (HEIGHT as f32);
    let near:   f32 = 0.1;
    let far:    f32 = 100.0;
    let f:      f32 = 1.0 / (fov / 2.0).tan();

    // Setting up the matrix that will introduce depth
    let perspective = vec![
        vec![f/aspect,  0.0,    0.0,                            0.0],
        vec![0.0,       f,      0.0,                            0.0],
        vec![0.0,       0.0,    (far + near) / (near - far),    (2.0 * far * near) / (near - far)],
        vec![0.0,       0.0,    -1.0,                           0.0]
    ];

    let mut prism_perspected: Vec<Vec<Vec<f32>>> = Vec::new();

    let mut prism_screen: Vec<(i32, i32)> = Vec::new();

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "Prism",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("Some error occured: {e}");
    });

    while window.is_open() {
        // Transformation
        prism = prism_original.clone(); // restore the original every frame

        // Increment angle
        theta += THETA_ITER;

        // Get the new rotation
        let rotation = vec![
            vec![theta.cos(),   0.0,    theta.sin(),    0.0],
            vec![0.0,           1.0,    0.0,            0.0],
            vec![-theta.sin(),  0.0,    theta.cos(),    0.0],
            vec![0.0,           0.0,    0.0,            1.0],
        ];

        // Apply transformation to all the points
        for i in 0..prism.len() {
            prism[i] = dot_product(&rotation, &prism[i]);
            prism[i][2][0] -= 5.0;  // Move point back by 5 units in Z
        }

        // Perspective

        // Clean previous points
        prism_perspected.clear();

        // Apply perspective to each vertex
        for i in 0..prism.len() {
            let mut temp = dot_product(&perspective, &prism[i]);
            
            // Fixing point in NDC
            temp[0][0] = temp[0][0] / temp[3][0]; // x
            temp[1][0] = temp[1][0] / temp[3][0]; // y
            temp[2][0] = temp[2][0] / temp[3][0]; // z
            
            // Drop the last element (useless)
            temp.remove(3);

            // prism_perspected only 3d coordinates
            prism_perspected.push(temp);
        }

        // Conversion to screen

        // Clean previous points
        prism_screen.clear();

        // Find the position of the vertex in the 2D window
        for i in 0..prism_perspected.len() {
            prism_screen.push(
                (
                    ((prism_perspected[i][0][0] + 1.0) * 0.5 * (WIDTH as f32)).floor() as i32,
                    ((1.0 - prism_perspected[i][1][0]) * 0.5 * (WIDTH as f32)).floor() as i32
                )
            );
        }

        // Drawing
        draw(&mut buffer, &prism_screen, &faces);

        // Update window
        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();

    }

}
