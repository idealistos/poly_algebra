# PolyAlgebra

A dynamic geometric construction and algebraic curve plotting tool that combines interactive geometry with computational algebra.

## Summary

PolyAlgebra is an interactive web application that allows users to create geometric constructions and visualize algebraic curves. The tool features:

- **Interactive Geometry**: Create points, lines, and geometric constructions through an intuitive canvas interface
- **Algebraic Computation**: Automatically generates and solves systems of polynomial equations from geometric constraints
- **Curve Visualization**: Plots locus curves with beautiful color-coded visualization
- **Scene Management**: Save and manage multiple geometric scenes
- **Real-time Updates**: Dynamic updates as you modify geometric constructions

The application consists of a Rust backend that handles algebraic computations and a React frontend that provides the interactive interface. The backend uses polynomial elimination techniques to solve geometric constraints and generate curve equations, while the frontend renders these as interactive visualizations.

## Demo

![PolyAlgebra Demo](Demo.png)

### Getting the Demo Running

1. **Start the Backend**:

   ```bash
   # Initialize the database (first time only)
   cargo run -- init

   # Start the web server
   cargo run -- start
   ```

2. **Start the Frontend**:

   ```bash
   cd frontend
   npm install
   npm run dev
   ```

3. **Open the Application**:

   - Navigate to `http://localhost:5174` in your browser
   - The application will load with a default scene

4. **Plot Your First Curve**:

The [Lemniscate of Bernoulli](https://en.wikipedia.org/wiki/Lemniscate_of_Bernoulli) is defined as the set of points for which the product of distances from this point to two "foci" is constant, equal to 1/4th of the square of the distance between the foci. To plot the lemniscate:

- Use the action ribbon on the left to add geometric objects:
  - Click "Fixed point" (1st action icon), then click the (-3, 0) grid point to create the point called "A"
  - Repeat this step to create the point "B" (3, 0)
  - Hover over "Fixed point" to make point actions appear, then Click "Free point" (2nd action icon in the group) to add the point "X" (0, 0)
- Hover over "d(•,•)" icon (2nd action icon from the bottom) to make invariant actions appear, then click the "*" icon (the last one in the group) to add a custom invariant, then enter "d(A, X) \* d(B, X)"
- Click "Locus" (the last action icon) and then choose the point "X" to indicate that we want to plot the set of all points satisfying the invariant for X
- Wait until the curve is displayed

## Install

### Prerequisites

- **Node.js & npm**: For running the React frontend

  - Download from [nodejs.org](https://nodejs.org/)
  - Required for package management and development server

- **Rust & Cargo**: For the backend algebraic computation engine

  - Install via [rustup.rs](https://rustup.rs/)
  - Handles polynomial operations, equation solving, and database management

- **Pari/GP**: For advanced algebraic computations
  - Download from [pari.math.u-bordeaux.fr](https://pari.math.u-bordeaux.fr/)
  - Used for complex polynomial elimination and algebraic operations
  - The application will automatically detect `gp` or `gp.exe` in your system PATH

### Installation Steps

1. **Clone the Repository**:

   ```bash
   git clone <repository-url>
   cd PolyAlgebra
   ```

2. **Install Frontend Dependencies**:

   ```bash
   cd frontend
   npm install
   ```

3. **Install Backend Dependencies**:

   ```bash
   # From the root directory
   cargo build
   ```

4. **Initialize the Database**:
   ```bash
   cargo run -- init
   ```

### Running the Application

1. **Start the Backend Server**:

   ```bash
   cargo run -- start
   ```

   The backend will start on `http://localhost:8080`

2. **Start the Frontend Development Server**:

   ```bash
   cd frontend
   npm run dev
   ```

   The frontend will start on `http://localhost:5174`

3. **Access the Application**:
   Open your browser and navigate to `http://localhost:5174`

### Command Line Options

The backend supports several command line options:

```bash
# Show help
cargo run -- --help
cargo run -- init --help
cargo run -- start --help

# Specify custom Pari/GP executable path
cargo run -- start --gp-executable /usr/local/bin/gp
cargo run -- start --gp-executable "C:\Program Files\Pari\gp.exe"
```

### Development

- **Backend Development**: The Rust backend uses Actix-web for the API server and Sea-ORM for database management
- **Frontend Development**: React with TypeScript, Konva for canvas rendering, and Material-UI for components
- **Database**: SQLite database with automatic migrations
- **Algebraic Engine**: Custom polynomial manipulation with Pari/GP integration

### Troubleshooting

- **Port Conflicts**: Ensure ports 8080 (backend) and 5174 (frontend) are available
- **Pari/GP Issues**:
  - Verify `gp` or `gp.exe` is accessible from command line
  - Use `--gp-executable` to specify custom path if auto-detection fails
- **Database Issues**: Delete `scenes.db` and re-run `cargo run -- init` to reset
- **Frontend Build Issues**: Clear `node_modules` and re-run `npm install`
