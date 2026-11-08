import { useQuery } from "@tanstack/react-query";

type SdqClient = {
  id: string;
  codeName: string;
  dateofbirth: string;
  gender: string;
  council: string;
  ethnicity: string;
  eal: string;
  disability: string;
  careexp: string;
  fundsrc: string;
};
type ClientApi = {
  clients: SdqClient[];
};

const EMPTY_CLIENTS: SdqClient[] = [];

const useClientApi = (): ClientApi => {
  const { data } = useQuery<SdqClient[]>({
    queryKey: ["clients"],
    queryFn: async () => {
      const response = await fetch("/api/clients");
      if (!response.ok) {
        throw new Error("Network response was not ok");
      }
      return response.json() as Promise<SdqClient[]>;
    },
  });

  return {
    clients: data || EMPTY_CLIENTS,
  };
};

export default useClientApi;
