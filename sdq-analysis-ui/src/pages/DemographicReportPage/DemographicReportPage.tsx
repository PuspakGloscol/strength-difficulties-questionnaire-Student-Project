import React from "react";
import { DEMOGRAPHIC_FIELDS, type DemographicField } from "@/api/types";
import useDemographicReport from "@/api/ClientApi/useDemographicReport";
import useReference from "@/api/ReferenceApi";

function DemographicReportPage() {
  const {
    data: { demographicFields },
    getLabelForDemographicValue,
  } = useReference();
  const options = DEMOGRAPHIC_FIELDS.filter((field) => field in demographicFields);
  const [demographicName, setDemographicName] =
    React.useState<DemographicField>("Gender");
  const { demographicReport } = useDemographicReport(demographicName);

  const onDemographicNameChange: React.ChangeEventHandler<HTMLSelectElement> = (
    e,
  ) => {
    setDemographicName(e.target.value as DemographicField);
  };

  return (
    <div>
      <h2>Demographic Report</h2>
      <p>
        This report lets you get a quick idea of the distribution of the various
        demographic values.
      </p>
      <div className="form-group">
        <label>Demographic</label>
        <select value={demographicName} onChange={onDemographicNameChange}>
          {options.map((field) => (
            <option key={field} value={field}>
              {field}
            </option>
          ))}
        </select>
      </div>

      <table>
        <thead>
          <tr>
            <th>Option</th>
            <th>Count</th>
            <th>Percentage</th>
          </tr>
        </thead>
        <tbody>
          {demographicReport.counts.map(({ option, count, percentage }) => (
            <tr key={option}>
              <td>{getLabelForDemographicValue(demographicName, option)}</td>
              <td>{count}</td>
              <td>{percentage} %</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

export default DemographicReportPage;
