import React from "react";

import ClientCard from "./ClientCard";
import ClientDeleteAllButton from "../ClientDeleteAllButton";
import ClientQueryForm from "@/components/ClientQueryForm";
import useSearchClients from "@/api/ClientApi/useSearchClients";
import {
  DEFAULT_CLIENT_QUERY,
  type ClientQueryDTO,
} from "@/api/ClientApi/clientApi";

function ClientListView() {
  const { data: clients, searchClients } = useSearchClients();
  const [clientQuery, setClientQuery] =
    React.useState<ClientQueryDTO>(DEFAULT_CLIENT_QUERY);

  const onClickQuery = () => {
    searchClients(clientQuery);
  };

  return (
    <div>
      <h2>
        Client List
        <span style={{ fontSize: "0.8em", marginLeft: "1em" }} />
        <ClientDeleteAllButton />
      </h2>
      <ClientQueryForm value={clientQuery} onChange={setClientQuery} />
      <button onClick={onClickQuery}>Search</button>

      {clients &&
        clients.map((client) => (
          <ClientCard key={client.clientId} client={client} />
        ))}
    </div>
  );
}

export default ClientListView;
