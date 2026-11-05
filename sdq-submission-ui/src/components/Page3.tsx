import useClientApi from "../api/clientApi";
import "./Nav.scss";

function Page3() {
  const { clients } = useClientApi();

  return (

    <div>
      <h2>Page 3</h2>
      {/*
      <p>This page demonstrates calling REST API for the simplest data</p>
      <p>Clients: {clients.length}</p>
      {clients.map((client) => (
        <div key={client.id}>
          <p>Client ID: {client.id}</p>
          <p>Client Name: {client.codeName}</p>
        </div>
    ))}*/}

        
      <div className="cltable">
        <table>
          <thead>
            <tr>
              <th>ID</th>
              <th>Code Name</th>
              <th>Date of Birth</th>
              <th>Gender</th>
              <th>Council</th>
              <th>Ethnicity</th>
              <th>EAL</th>
              <th>Disability Status</th>
              <th>Care Experience</th>
              <th>Funding Source</th>
            </tr>
          </thead>
          <tbody>
            {clients.map((client) => (
              <tr>
                <th>{client.id}</th>
                <th>{client.codeName}</th>
                <th>{client.dateofbirth}</th>
                <th>{client.gender}</th>
                <th>{client.council}</th>
                <th>{client.ethnicity}</th>
                <th>{client.eal}</th>
                <th>{client.disability}</th>
                <th>{client.careexp}</th>
                <th>{client.fundsrc}</th>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

    </div>
  );
}

export default Page3;
