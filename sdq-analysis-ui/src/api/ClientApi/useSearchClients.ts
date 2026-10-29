import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import useInProgressContext from "@/context/InProgressContext";
import useAppNotificationContext from "@/context/AppNotificationContext";
import {
  BASE_CLIENT_URL,
  parseFile,
  type ClientFile,
  type ClientQueryDTO,
} from "@/api/ClientApi/clientApi";

function useSearchClients() {
  const { addMessage } = useAppNotificationContext();
  const { beginJob, endJob } = useInProgressContext();
  const [searchRequest, setSearchRequest] = useState<{
    query: ClientQueryDTO | null;
    searchId: number;
  }>({ query: null, searchId: 0 });

  const result = useQuery<ClientFile[], Error>({
    queryKey: ["clientFiles", searchRequest.searchId, searchRequest.query],
    enabled: searchRequest.query !== null,
    queryFn: async () => {
      if (searchRequest.query === null) {
        throw new Error("A client query is required");
      }

      const { query } = searchRequest;
      const jobId = beginJob("Fetching files");

      try {
        const response = await fetch(`${BASE_CLIENT_URL}/search`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(query),
        });

        if (!response.ok) {
          addMessage(
            "danger",
            response.status,
            "Failed to fetch file: " + response.statusText,
          );
          throw new Error("Network response was not ok");
        }

        const json = await response.json();
        return json.map(parseFile);
      } finally {
        endJob(jobId);
      }
    },
  });

  const searchClients = (query: ClientQueryDTO) => {
    setSearchRequest((prev) => ({ query, searchId: prev.searchId + 1 }));
  };

  return { ...result, searchClients };
}

export default useSearchClients;
