import React from "react";
import { useNavigate } from "react-router";

type Props = {
  clientId: string;
};

const ClientViewDetailsButton: React.FC<Props> = ({ clientId }) => {
  const navigate = useNavigate();

  const onClickViewDetails = () => {
    navigate(`/client/${clientId}`);
  };

  return (
    <button
      type="button"
      className="client-card__button"
      onClick={onClickViewDetails}
    >
      View details
    </button>
  );
};

export default ClientViewDetailsButton;
