import cn from 'classnames';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import z from 'zod';
import cls from './form-minting-settings.module.css';
import type { MintingSettings} from '@/shared/store/mint-store';
import { useMintStore } from '@/shared/store/mint-store';
import { InputControlled } from '@/shared/ui/input';

interface FormMintingSettingsProps {
    className?: string;
}

type FormMintingSettingsValues = MintingSettings

const schema = z.object({
    premint: z.string().regex(/^\d+$/, 'Must be a positive integer').optional(),
    'maximum tokens': z
        .string()
        .regex(/^\d+$/, 'Must be a positive integer')
        .optional(),
});

export function FormMintingSettings(props: FormMintingSettingsProps) {
    const { className } = props;
    const setMintingSettings = useMintStore(
        (state) => state.setMintingSettings,
    );
    const mintingSettings = useMintStore((state) => state.mintingSettings);

    const { handleSubmit, control } = useForm<FormMintingSettingsValues>({
        values: mintingSettings,
        shouldFocusError: false,
        resolver: zodResolver(schema),
    });

    function onAction(data: FormMintingSettingsValues) {
        console.log(data);
        setMintingSettings(data);
    }

    // const formProps = { register, errors };

    return (
        <form
            onChange={handleSubmit(onAction)}
            className={cn(className, cls.formIdentity)}
        >
            <div className='flex justify-between gap-4'>
                <InputControlled
                    {...{
                        control,
                        name: 'premint',
                        type: 'number',
                    }}
                />
                <InputControlled
                    {...{
                        control,
                        name: 'maximum tokens',
                        type: 'number',
                    }}
                />
            </div>
        </form>
    );
}