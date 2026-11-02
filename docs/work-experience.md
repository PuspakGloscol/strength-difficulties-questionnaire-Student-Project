# Work Experience

For those contributing to this as a work experience project, the current developer docs assume a fair bit of knowledge about software projects in general. This page will expand on some of the underlying technology.

## Source Control

Code changes are tracked using Git, a distributed version control system. Git allows multiple developers to work on the same codebase simultaneously, managing changes and resolving conflicts. It is essential to understand basic Git commands such as `clone`, `commit`, `push`, and `pull` to effectively contribute to the project.

One good resource for learning Git branching is [Learn Git Branching](https://learngitbranching.js.org/).

You can interact with Git largely within VS Code, in the long term I would recommend learning the command line interface for Git, as it is more powerful and flexible.

## Development Machine

I have developed this project within two different environments

- Windows Subsystem for Linux (WSL)
- MacOS

Whilst it should technically be possible to develop this natively on Windows, I have not tested this and cannot guarantee that it will work. If you are using Windows, I recommend using WSL to run a Linux environment.

To install WSL, it should be possible to find Ubuntu on the Microsoft Store. Once installed, you can open a terminal and run Linux commands directly from Windows.

### Linux Root Access

In Linux some commands require root access. You can run commands as root by prefixing them with `sudo`. You will be prompted for your password when using `sudo` for the first time in a session. Sudo means 'switcher user and do', and it allows you to run commands with elevated privileges. Be cautious when using `sudo`, as it can modify system files and settings.

## Docker

Docker is a technology that allows applications to be packaged and run in isolated environments called containers. This ensures that the application runs consistently across different machines and environments. Docker is used in this project to manage dependencies and simplify the setup process.

If you are using WSL, you can

- install Docker inside WSL directly
- or install Docker Desktop for Windows, which integrates with WSL.

For MacOS, you can install Docker Desktop for Mac.

If you install Docker on Linux, you may have to do a few extra steps to allow your user to run Docker commands without `sudo`. These steps will be outlined in the Docker installation instructions for your specific Linux distribution.

### First Steps in Docker

I recommend first running Dockers own hello world project to ensure that Docker is installed correctly. You can do this by running the following command in your terminal:

```bash
docker run hello-world
```

Then try running a simple web server (nginx). Run the following command:

```bash
docker run -d -p 8080:80 nginx
```

This shows port forwarding, and it should be possible to open a web browser and navigate to `http://localhost:8080` to see the nginx welcome page.

## React

Both frontends are written in React, using TypeScript which is compiled to JavaScript. The projects are bootstrapped using [Vite](https://vitejs.dev/), which is a modern frontend build tool that provides fast development and optimized production builds.

In addition to Vite, I use

- [React Router](https://reactrouter.com/) for client-side routing
- [React Query](https://tanstack.com/query/latest) for data fetching and caching

## Clean Architecture

Each API project uses a 'clean architecture' approach. This means that each aspect of the system is in it's own module, and then there is a single main entry point which brings everything together.

For example, in the Submission API, written in Go. The `app/app.go` contains the entry point, but it refers to the REST API which is defined in `adapters/rest_api/rest_api.go`.
It also refers to the domain model and service which is defined in `domain/sdq_service.go`.

The Rust and Java APIs are structured in a similar way.

## Exploratory Tasks

To get started, try the following 'hello world' tasks.

- Edit the text that comes back from the health endpoint of the submission API. This is defined in `adapters/rest_api/rest_api.go` in the `NewRouter` function.
- Add a paragraph to the first page of the Submission UI. Start looking in `sdq-submission-ui/src/App.tsx`.
