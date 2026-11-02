import React from "react";
import type { GoalProgress } from "@/api/GboApi/gboApi";
import useReference from "@/api/ReferenceApi/useReference";
import ClientLink from "@/components/ClientLink";

interface Props {
  goals?: GoalProgress[];
}

function progressString(firstScore: number, lastScore: number): string {
  const progress = lastScore - firstScore;
  if (progress > 0) {
    return `+${progress}`;
  } else if (progress < 0) {
    return `-${progress}`;
  } else {
    return "0";
  }
}

const GoalProgressTable: React.FC<Props> = ({ goals }) => {
  const { getLabelForGoalType } = useReference();

  if (!goals) {
    return <div>No goal progress found.</div>;
  }

  return (
    <table>
      <thead>
        <tr>
          <th>Client ID</th>
          <th>Type</th>
          <th>Description</th>
          <th>Assessor</th>
          <th>First Score</th>
          <th>Last Score</th>
          <th>Progress</th>
        </tr>
      </thead>
      <tbody>
        {goals.map((goal) => (
          <tr key={goal.goal.goalId}>
            <td>
              <ClientLink clientId={goal.goal.clientId} />
            </td>
            <td>{getLabelForGoalType(goal.goal.type)}</td>
            <td>{goal.goal.description}</td>
            <td>{goal.assessor}</td>
            <td>{goal.firstScore}</td>
            <td>{goal.lastScore}</td>
            <td>{progressString(goal.firstScore, goal.lastScore)}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
};

export default GoalProgressTable;
