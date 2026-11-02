import React from "react";
import useHealthApi from "../api/healthApi";

function Page2() {
  const { greet } = useHealthApi();
  const [greeting, setGreeting] = React.useState<string | null>(null);

  const onSubmit = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const formData = new FormData(event.currentTarget);
    const name = formData.get("name") as string;
    greet(name)
      .then(setGreeting)
      .catch((error) => {
        console.error("Error greeting:", error);
        alert("Failed to greet. Please try again.");
      });
  };

  return (
    <>
      <h2>Page 2</h2>
      <p>This is a page just for demonstration purposes.</p>
      <p>
        It shows the most basic pulling of data from a form, submitting to REST
        API and getting a response back.
      </p>
      <form onSubmit={onSubmit}>
        <label>
          Name:
          <input type="text" name="name" required />
        </label>
        <button type="submit">Greet</button>
      </form>
      {greeting && <p>Greeting: {greeting}</p>}
    </>
  );
}

export default Page2;
