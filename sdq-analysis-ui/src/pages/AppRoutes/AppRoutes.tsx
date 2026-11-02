import { Routes, Route } from "react-router";
import ClientFilePage from "@/pages/ClientFilePage";
import BulkUploadPage from "@/pages/BulkUploadPage";
import HomePage from "@/pages/HomePage";
import DemographicReportPage from "@/pages/DemographicReportPage";
import GoalBasedOutcomePage from "@/pages/GoalBasedOutcomePage";
import SdqResponsePage from "@/pages/SdqResponsePage";
import AppLayout from "@/AppLayout";

const AppRoutes = () => {
  return (
    <Routes>
      <Route element={<AppLayout />}>
        <Route path="/" element={<HomePage />} />
        <Route path="/demographic-report" element={<DemographicReportPage />} />
        <Route path="/gbo" element={<GoalBasedOutcomePage />} />
        <Route path="/sdq" element={<SdqResponsePage />} />
        <Route path="/client/:id" element={<ClientFilePage />} />
        <Route path="/bulk-upload" element={<BulkUploadPage />} />
      </Route>
    </Routes>
  );
};

export default AppRoutes;
