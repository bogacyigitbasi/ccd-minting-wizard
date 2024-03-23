import { Controller } from 'react-hook-form';
import * as React from 'react';
import { Text } from '../../text';
import { ErrorMessage } from '../../error-message';
import { Input } from './input.shadcn';
import type { Control, FieldValues, Path } from 'react-hook-form';

export interface InputControlledProps<T extends FieldValues>
    extends React.InputHTMLAttributes<HTMLInputElement> {
    control: Control<T>;
    name: Path<T>;
    rules?: Parameters<typeof Controller<T>>[0]['rules'];
    labeled?: boolean;
    label?: string;
}

export function InputControlled<T extends FieldValues>(
    props: InputControlledProps<T>,
) {
    const {
        control,
        name,
        rules,
        labeled = true,
        className,
        label = name,
        onChange,
        ...otherProps
    } = props;

    return (
        <Controller
            control={control}
            name={name}
            rules={rules}
            render={({ field, fieldState: { error } }) => {
                return (
                    <div className={className}>
                        {labeled && (
                            <Text
                                className='capitalize'
                                text={label}
                                size={'xs'}
                            />
                        )}
                        <Input
                            {...field}
                            {...otherProps}
                            onChange={(e) => {
                                field.onChange(e);
                                onChange && onChange(e);
                            }}
                            placeholder={`add ${label}...`}
                        />
                        <ErrorMessage message={error?.message} />
                    </div>
                );
            }}
        />
    );
}
