import { useQuery } from "@tanstack/react-query";

interface HealthApi {
  healthMessage: string;
  greet: (name: string) => Promise<string>;
}

const DEFAULT_HEALTH_MESSAGE = "unknown";

const useHealthApi = (): HealthApi => {
  // This is a simple health check endpoint on /api/health
  const { data } = useQuery({
    queryKey: ["health"],
    queryFn: async () => {
      const response = await fetch("/api/health");
      if (!response.ok) {
        throw new Error("Network response was not ok");
      }
      return response.text();
    },
  });

  const greet = async (name: string): Promise<string> => {
    const response = await fetch("/api/greet", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ name }),
    });
    if (!response.ok) {
      throw new Error("Network response was not ok");
    }
    const result: { name: string; greeting: string } = await response.json();
    return result.greeting;
  };

  return {
    healthMessage: data || DEFAULT_HEALTH_MESSAGE,
    greet,
  };
};

export default useHealthApi;
