import useHealthApi from "../api/healthApi";

function Page1() {
  const { healthMessage } = useHealthApi();
  return (
    <>
      <h2>Page 1</h2>
      <p>This is a page just for demonstration purposes.</p>
      <p>
        It shows the simplest use of a GET query on the REST API, and putting
        the response back out to the user.
      </p>
      <p>Health API Response: {healthMessage}</p>
    </>
  );
}

export default Page1;
