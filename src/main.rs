#[macro_use]
extern crate glium;
extern crate time;
extern crate rand;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;

const PROJECTION_MATRIX: [[f32; 3]; 3]= [
    [ 2.0 / WIDTH as f32, 0.0,                 0.0],
    [ 0.0,               -2.0 / HEIGHT as f32, 0.0],
    [-1.0,                1.0,                 1.0]
];

const BAT_SPEED: f32 = 300f32;
const BALL_SPEED: f32 = 200f32;

struct Vector {
    x: f32,
    y: f32
}

impl Vector {
    fn new(x: f32, y: f32) -> Vector {
        Vector {
            x: x,
            y: y
        }
    }

    fn set(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    fn setX(&mut self, x: f32) {
        self.x = x;
    }

    fn setY(&mut self, y: f32) {
        self.y = y;
    }

    fn add(&mut self, v: &Vector) {
        self.x += v.x;
        self.y += v.y;
    }
}

#[derive(Copy, Clone)]
struct Vertex {
    position: [i32; 2]
}
implement_vertex!(Vertex, position);

struct Entity {
    w:             u32,
    h:             u32,
    hw:            u32,
    hh:            u32,
    position:      Vector,
    velocity:      Vector,
    vertex_buffer: glium::vertex::VertexBuffer<Vertex>,
    mv_matrix:     [[f32; 3]; 3]
}


impl Entity {
    fn new(display: &glium::backend::glutin_backend::GlutinFacade, w: u32, h: u32) -> Entity {
        Entity {
            w: w,
            h: h,

            hw: w / 2,
            hh: h / 2,

            position: Vector::new(0f32, 0f32),

            velocity: Vector::new(0f32, 0f32),

            vertex_buffer: glium::VertexBuffer::new(display, &get_rekt(w, h)).unwrap(),

            mv_matrix: get_identity()
        }
    }

    fn update(&mut self, dt: &f32) {
        self.position.x += self.velocity.x * dt;
        self.position.y += self.velocity.y * dt;

        self.mv_matrix[2][0] = self.position.x + self.hw as f32;
        self.mv_matrix[2][1] = self.position.y + self.hh as f32;
    }

    fn draw(&self, frame: &mut glium::Frame, program: &glium::Program) {
        use glium::Surface;

        let uniforms = uniform! {
            mvMatrix: self.mv_matrix,
            projectionMatrix: PROJECTION_MATRIX
        };
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);

        frame.draw(&self.vertex_buffer, &indices, program, &uniforms, &Default::default()).unwrap();
    }
}

fn get_rekt(w: u32, h: u32) -> Vec<Vertex> {
    let hw = w as i32 / 2;
    let hh = h as i32 / 2;

    return vec![
        Vertex { position: [-hw, -hh] },
        Vertex { position: [-hw, hh]  },
        Vertex { position: [hw, -hh]  },
        Vertex { position: [hw, hh]   }
    ]
}

fn cheeky_collision(e1: &Entity, e2: &Entity) -> bool {
    let e1_max = Vector {
        x: e1.position.x + e1.w as f32,
        y: e1.position.y + e1.h as f32
    };

    let e2_max = Vector {
        x: e2.position.x + e2.w as f32,
        y: e2.position.y + e2.h as f32
    };

    !(e1_max.x < e2.position.x ||
      e1_max.y < e2.position.y ||
      e1.position.x > e2_max.x ||
      e1.position.y > e2_max.y)
}

fn main() {
    use glium::{DisplayBuild, Surface};
    let display = glium::glutin::WindowBuilder::new()
        .with_dimensions(WIDTH, HEIGHT)
        .with_title(format!("Rust Pong"))
        .build_glium()
        .unwrap();

    let vertex_shader_src = r#"
        #version 140

        in vec2 position;

        uniform mat3 mvMatrix;
        uniform mat3 projectionMatrix;

        void main() {
            gl_Position = vec4(projectionMatrix * mvMatrix * vec3(position, 1.0), 1.0);
        }
    "#;

    let fragment_shader_src = r#"
        #version 140

        out vec4 color;

        void main() {
            color = vec4(1.0, 1.0, 1.0, 1.0);
        }
    "#;

    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

    let mut ball = Entity::new(&display, 10, 10);
    let mut cpu = Entity::new(&display, 100, 20);
    let mut player = Entity::new(&display, 100, 20);

    player.position.setY((HEIGHT - player.h) as f32);
    ball.position.set((WIDTH / 2) as f32, (HEIGHT / 2) as f32);

    ball.velocity.setY(BALL_SPEED);

    let mut dt = 0.0;

    loop {
        let begin = time::PreciseTime::now();

        for ev in display.poll_events() {
            use glium::glutin::Event;
            use glium::glutin::ElementState::Pressed;
            use glium::glutin::VirtualKeyCode;

            match ev {
                Event::KeyboardInput(Pressed, any, Some(VirtualKeyCode::Left)) => {
                    player.velocity.setX(-BAT_SPEED)
                },
                Event::KeyboardInput(Pressed, any, Some(VirtualKeyCode::Right)) => {
                    player.velocity.setX(BAT_SPEED);
                },
                Event::KeyboardInput(Released, any, Some(VirtualKeyCode::Left)) => {
                    player.velocity.setX(0f32)
                },
                Event::KeyboardInput(Released, any, Some(VirtualKeyCode::Right)) => {
                    player.velocity.setX(0f32);
                },
                Event::Closed => return,
                _ => ()
            }
        }

        if ball.position.x > (cpu.position.x + (cpu.hw / 2) as f32) &&
           ball.position.x < ((cpu.position.x + cpu.w as f32) - (cpu.hw / 2) as f32) {
                cpu.velocity.setX(0f32);
        } 
        else if ball.position.x > (cpu.position.x + cpu.hw as f32) {
            cpu.velocity.setX(BAT_SPEED);
        } 
        else if (ball.position.x + ball.w as f32) < cpu.position.x {
            cpu.velocity.setX(-BAT_SPEED);
        }

        if cheeky_collision(&ball, &cpu) {
            ball.velocity.set(get_random_vel(), BALL_SPEED + get_random_vel());
        }

        if cheeky_collision(&ball, &player) {
            ball.velocity.set(get_random_vel(), -BALL_SPEED + get_random_vel());
        }

        if ball.position.x < 0f32 {
            ball.velocity.setX(BALL_SPEED)
        }
        else if ball.position.x + ball.w as f32 > WIDTH as f32 {
            ball.velocity.setX(-BALL_SPEED)
        }

        if ball.position.y < 0f32 {
            ball.position.set((WIDTH / 2) as f32, (HEIGHT / 2 - ball.h) as f32);
            ball.velocity.setX(get_random_vel());
        } 
        else if (ball.position.y + ball.h as f32) >  HEIGHT as f32 {
            ball.position.set((WIDTH / 2) as f32, (HEIGHT / 2 - ball.h) as f32);
            ball.velocity.setX(get_random_vel());
        }

        ball.update(&dt);
        cpu.update(&dt);
        player.update(&dt);

        let mut frame = display.draw();

        frame.clear_color(0.0, 0.0, 0.0, 1.0);

        cpu.draw(&mut frame, &program);
        player.draw(&mut frame, &program);
        ball.draw(&mut frame, &program);

        frame.finish().unwrap();

        dt = begin.to(time::PreciseTime::now()).num_milliseconds() as f32 / 1000.0;
    }
}

fn get_identity() -> [[f32; 3]; 3] {
    return [
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0]
    ];
}

fn get_random_vel() -> f32 {
    let x_speed = rand::random::<f32>() * BALL_SPEED;
    let negative_chance = rand::random::<f32>();

    if negative_chance > 0.5 {
        return x_speed * -1.0;
    }

    return x_speed;
}
