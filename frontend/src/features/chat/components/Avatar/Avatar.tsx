import React from 'react';
import { AVATAR, CSS_CLASSES } from '../../constants';
import { buildClassName } from '../../utils';

interface AvatarProps {
  type: 'user' | 'assistant';
}

const Avatar: React.FC<AvatarProps> = ({ type }) => {
  const label = type === 'user' ? AVATAR.USER : AVATAR.ASSISTANT;
  const testId = `${type}-avatar`;
  const className = buildClassName(CSS_CLASSES.AVATAR, type);
  
  return (
    <div data-testid={testId} className={className}>
      {label}
    </div>
  );
};

export default Avatar;