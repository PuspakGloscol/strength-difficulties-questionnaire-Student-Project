import useClientApi from "../api/clientApi";

function Page3() {
  const { clients } = useClientApi();

  return (
    <div>
      <h2>Page 3</h2>
      <p>This page demonstrates calling REST API for the simplest data</p>
      <p>Clients: {clients.length}</p>
      {clients.map((client) => (
        <div key={client.id}>
          <p>Client ID: {client.id}</p>
          <p>Client Name: {client.name}</p>
        </div>
      ))}
    </div>
  );
}

export default Page3;
