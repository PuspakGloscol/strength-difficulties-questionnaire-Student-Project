import Nav from "./components/Nav";

function App({ children }: { children: React.ReactNode }) {
  return (
    <main>
      <h1>SDQ Submission UI</h1>
      <Nav />
      {children}
    </main>
  );
}

export default App;
