# Agent Usage Documentation

This document summarizes everything you need to get started with the agent, including how to run tests, start the project, and validate Kubernetes manifests and Dockerfiles.

## Running Tests
- **Unit and Integration Tests**:  
    Run tests with:
    ```
    cargo test
    ```

## Starting the Project
- **Development Mode**:  
    Run the project locally with:
    ```
    cargo run
    ```
- **Production Mode**:  
    Build in release mode and run the executable:
    ```
    cargo build --release && ./target/release/your_project_executable
    ```

## Validating Kubernetes Manifests
- **Using kubeval**:  
    Validate your Kubernetes manifests using [kubeval](https://github.com/instrumenta/kubeval):
    ```
    kubeval path/to/manifest.yaml
    ```
- **Using kube-linter**:  
    Alternatively, lint your manifests with:
    ```
    kube-linter lint path/to/manifests/
    ```

## Validating Dockerfile
- **Using hadolint**:  
    Validate your Dockerfile with [hadolint](https://github.com/hadolint/hadolint):
    ```
    hadolint Dockerfile
    ```
- **Using dockerfilelint**:  
    For another option, use dockerfilelint:
    ```
    dockerfilelint Dockerfile
    ```

## Additional Tips
- **Environment Variables**:  
    Ensure you have set all necessary environment variables; refer to the `.env.example` file for guidance.
- **Continuous Integration (CI)**:  
    Integrate these commands in your CI pipeline to automate tests and validations.
- **Documentation**:  
    Check the README.md in the project repository for further instructions and project-specific details.

Happy coding!
