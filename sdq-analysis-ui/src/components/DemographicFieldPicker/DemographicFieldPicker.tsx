import React from "react";
import { DEMOGRAPHIC_FIELDS, type DemographicField } from "@/api/types";
import useReference from "@/api/ReferenceApi";

interface Props {
  value: DemographicField;
  onChange: (v: DemographicField) => void;
}

const DemographicFieldPicker: React.FC<Props> = ({ value, onChange }) => {
  const {
    data: { demographicFields },
  } = useReference();
  const options = DEMOGRAPHIC_FIELDS.filter((field) => field in demographicFields);

  const onInternalChange: React.ChangeEventHandler<HTMLSelectElement> = ({
    target: { value },
  }) => {
    onChange(value as DemographicField);
  };

  return (
    <select value={value} onChange={onInternalChange}>
      {options.map((option) => (
        <option key={option} value={option}>
          {option}
        </option>
      ))}
    </select>
  );
};

export default DemographicFieldPicker;
